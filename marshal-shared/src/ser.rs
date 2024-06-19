use std::any::Any;
use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::ops::{CoerceUnsized, Deref};
use std::{mem, rc, sync};
use weak_table::traits::{WeakElement, WeakKey};
use weak_table::PtrWeakKeyHashMap;

use marshal::context::Context;
use marshal::encode::{AnyEncoder, Encoder};
use marshal::reexports::anyhow;
use marshal::ser::Serialize;
use marshal::Serialize;
use marshal_pointer::arc_ref::ArcRef;
use marshal_pointer::arc_weak_ref::ArcWeakRef;
use marshal_pointer::rc_ref::RcRef;
use marshal_pointer::rc_weak_ref::RcWeakRef;
use marshal_pointer::{AsFlatRef, DerefRaw};

struct ByAddress<T>(T);

impl<T: DerefRaw> Hash for ByAddress<T> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.0.deref_raw().hash(state)
    }
}

impl<T: DerefRaw> PartialEq<Self> for ByAddress<T> {
    fn eq(&self, other: &Self) -> bool {
        (self.0.deref_raw() as *const ()).eq(&(other.0.deref_raw() as *const ()))
    }
}

impl<T: DerefRaw> Eq for ByAddress<T> {}

struct PointerState {
    id: usize,
    written: bool,
}

pub struct SharedSerializeContext<WeakAny> {
    next_id: usize,
    shared: HashMap<ByAddress<WeakAny>, PointerState>,
}

impl<WeakAny> Default for SharedSerializeContext<WeakAny> {
    fn default() -> Self {
        SharedSerializeContext {
            next_id: 0,
            shared: HashMap::new(),
        }
    }
}

impl<WeakAny: 'static + DerefRaw> SharedSerializeContext<WeakAny> {
    fn get_state(ctx: &mut Context, weak: WeakAny) -> &mut PointerState {
        let this = ctx.get_or_default::<Self>();
        this.shared.entry(ByAddress(weak)).or_insert_with(|| {
            let state = PointerState {
                id: this.next_id,
                written: false,
            };
            this.next_id += 1;
            state
        })
    }
    fn serialize_strong<T: Serialize<E>, E: Encoder>(
        value: &T,
        weak: WeakAny,
        e: AnyEncoder<E>,
        ctx: &mut Context,
    ) -> anyhow::Result<()> {
        let state = Self::get_state(ctx, weak);
        let id = state.id;
        let written = mem::replace(&mut state.written, true);
        Shared::<T> {
            id,
            inner: (!written).then_some(value),
        }
        .serialize(e, ctx)
    }
    fn serialize_weak<E: Encoder>(
        weak: WeakAny,
        e: AnyEncoder<E>,
        ctx: &mut Context,
    ) -> anyhow::Result<()> {
        let state = Self::get_state(ctx, weak);
        let id = state.id;
        id.serialize(e, ctx)
    }
}

#[derive(Serialize)]
struct Shared<'a, T> {
    id: usize,
    inner: Option<&'a T>,
}

pub fn serialize_rc<E: Encoder, T: 'static + Serialize<E>>(
    ptr: &RcRef<T>,
    e: AnyEncoder<'_, E>,
    ctx: &mut Context,
) -> anyhow::Result<()> {
    SharedSerializeContext::<rc::Weak<dyn Any>>::serialize_strong(&**ptr, ptr.weak(), e, ctx)
}

pub fn serialize_arc<E: Encoder, T: 'static + Sync + Send + Serialize<E>>(
    ptr: &ArcRef<T>,
    e: AnyEncoder<'_, E>,
    ctx: &mut Context,
) -> anyhow::Result<()> {
    SharedSerializeContext::<sync::Weak<dyn Any>>::serialize_strong(&**ptr, ptr.weak(), e, ctx)
}

pub fn serialize_rc_weak<E: Encoder, T: 'static + Serialize<E>>(
    ptr: &RcWeakRef<T>,
    e: AnyEncoder<'_, E>,
    ctx: &mut Context,
) -> anyhow::Result<()> {
    SharedSerializeContext::<rc::Weak<dyn Any>>::serialize_weak(ptr.weak(), e, ctx)
}

pub fn serialize_arc_weak<E: Encoder, T: 'static + Sync + Send + Serialize<E>>(
    ptr: &ArcWeakRef<T>,
    e: AnyEncoder<'_, E>,
    ctx: &mut Context,
) -> anyhow::Result<()> {
    SharedSerializeContext::<sync::Weak<dyn Any>>::serialize_weak(ptr.weak(), e, ctx)
}

#[macro_export]
macro_rules! derive_serialize_rc_shared {
    ($ty:ty) => {
        impl<E: $crate::reexports::marshal::encode::Encoder>
            $crate::reexports::marshal::ser::rc::SerializeRc<E> for $ty
        {
            fn serialize_rc(
                this: &$crate::reexports::marshal_pointer::rc_ref::RcRef<Self>,
                e: $crate::reexports::marshal::encode::AnyEncoder<'_, E>,
                ctx: &mut $crate::reexports::marshal::context::Context,
            ) -> anyhow::Result<()> {
                $crate::ser::serialize_rc::<E, Self>(this, e, ctx)
            }
        }
    };
}

#[macro_export]
macro_rules! derive_serialize_arc_shared {
    ($ty:ty) => {
        impl<E: $crate::reexports::marshal::encode::Encoder>
            $crate::reexports::marshal::ser::rc::SerializeArc<E> for $ty
        {
            fn serialize_arc(
                this: &$crate::reexports::marshal_pointer::arc_ref::ArcRef<Self>,
                e: $crate::reexports::marshal::encode::AnyEncoder<'_, E>,
                ctx: &mut $crate::reexports::marshal::context::Context,
            ) -> anyhow::Result<()> {
                $crate::ser::serialize_arc::<E, Self>(this, e, ctx)
            }
        }
    };
}

#[macro_export]
macro_rules! derive_serialize_rc_weak_shared {
    ($ty:ty) => {
        impl<E: $crate::reexports::marshal::encode::Encoder>
            $crate::reexports::marshal::ser::rc::SerializeRcWeak<E> for $ty
        {
            fn serialize_rc_weak(
                this: &$crate::reexports::marshal_pointer::rc_weak_ref::RcWeakRef<Self>,
                e: $crate::reexports::marshal::encode::AnyEncoder<'_, E>,
                ctx: &mut $crate::reexports::marshal::context::Context,
            ) -> anyhow::Result<()> {
                $crate::ser::serialize_rc_weak::<E, Self>(this, e, ctx)
            }
        }
    };
}

#[macro_export]
macro_rules! derive_serialize_arc_weak_shared {
    ($ty:ty) => {
        impl<E: $crate::reexports::marshal::encode::Encoder>
            $crate::reexports::marshal::ser::rc::SerializeArcWeak<E> for $ty
        {
            fn serialize_arc_weak(
                this: &$crate::reexports::marshal_pointer::arc_weak_ref::ArcWeakRef<Self>,
                e: $crate::reexports::marshal::encode::AnyEncoder<'_, E>,
                ctx: &mut $crate::reexports::marshal::context::Context,
            ) -> anyhow::Result<()> {
                $crate::ser::serialize_arc_weak::<E, Self>(this, e, ctx)
            }
        }
    };
}
