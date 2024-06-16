use std::{rc, sync};
use std::any::Any;
use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::hash::{Hash, Hasher};

use marshal::context::Context;
use marshal::encode::Encoder;
use marshal::reexports::anyhow;
use marshal::ser::Serialize;
use marshal::Serialize;
use marshal_pointer::arc_ref::ArcRef;
use marshal_pointer::arc_weak_ref::ArcWeakRef;
use marshal_pointer::DerefRaw;
use marshal_pointer::rc_ref::RcRef;
use marshal_pointer::rc_weak_ref::RcWeakRef;

struct ByAddress<T>(T);

impl<T: DerefRaw> Hash for ByAddress<T> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.0.deref_raw().hash(state)
    }
}

impl<T: DerefRaw> PartialEq<Self> for ByAddress<T> {
    fn eq(&self, other: &Self) -> bool {
        self.0.deref_raw().eq(&other.0.deref_raw())
    }
}

impl<T: DerefRaw> Eq for ByAddress<T> {}

#[derive(Default)]
pub struct SharedRcSerializeContext {
    next_id: usize,
    shared: HashMap<ByAddress<rc::Weak<dyn Any>>, usize>,
}

#[derive(Default)]
pub struct SharedArcSerializeContext {
    next_id: usize,
    shared: HashMap<ByAddress<sync::Weak<dyn Sync + Send + Any>>, usize>,
}

#[derive(Serialize)]
struct Shared<'a, T> {
    id: usize,
    inner: Option<&'a T>,
}

pub fn serialize_rc<E: Encoder, T: 'static + Serialize<E>>(
    ptr: &RcRef<T>,
    e: E::AnyEncoder<'_>,
    ctx: &mut Context,
) -> anyhow::Result<()> {
    let shared_ctx = ctx.get_or_default::<SharedRcSerializeContext>();
    match shared_ctx.shared.entry(ByAddress(ptr.weak())) {
        Entry::Occupied(entry) => {
            Shared::<T> {
                id: *entry.get(),
                inner: None,
            }
            .serialize(e, ctx)?;
            Ok(())
        }
        Entry::Vacant(entry) => {
            let id = shared_ctx.next_id;
            entry.insert(id);
            shared_ctx.next_id += 1;
            Shared {
                id,
                inner: Some(&**ptr),
            }
            .serialize(e, ctx)?;
            Ok(())
        }
    }
}

pub fn serialize_arc<E: Encoder, T: 'static + Sync + Send + Serialize<E>>(
    ptr: &ArcRef<T>,
    e: E::AnyEncoder<'_>,
    ctx: &mut Context,
) -> anyhow::Result<()> {
    let shared_ctx = ctx.get_or_default::<SharedArcSerializeContext>();
    match shared_ctx.shared.entry(ByAddress(ptr.weak())) {
        Entry::Occupied(entry) => {
            Shared::<T> {
                id: *entry.get(),
                inner: None,
            }
            .serialize(e, ctx)?;
            Ok(())
        }
        Entry::Vacant(entry) => {
            let id = shared_ctx.next_id;
            entry.insert(id);
            shared_ctx.next_id += 1;
            Shared {
                id,
                inner: Some(&**ptr),
            }
            .serialize(e, ctx)?;
            Ok(())
        }
    }
}

pub fn serialize_rc_weak<E: Encoder, T: 'static + Serialize<E>>(
    ptr: &RcWeakRef<T>,
    e: E::AnyEncoder<'_>,
    ctx: &mut Context,
) -> anyhow::Result<()> {
    let shared_ctx = ctx.get_or_default::<SharedRcSerializeContext>();
    let index = match shared_ctx.shared.entry(ByAddress(ptr.weak())) {
        Entry::Occupied(entry) => *entry.get(),
        Entry::Vacant(entry) => {
            let id = shared_ctx.next_id;
            entry.insert(id);
            shared_ctx.next_id += 1;
            id
        }
    };
    <usize as Serialize<E>>::serialize(&index, e, ctx)
}

pub fn serialize_arc_weak<E: Encoder, T: 'static + Sync+Send+Serialize<E>>(
    ptr: &ArcWeakRef<T>,
    e: E::AnyEncoder<'_>,
    ctx: &mut Context,
) -> anyhow::Result<()> {
    let shared_ctx = ctx.get_or_default::<SharedArcSerializeContext>();
    let index = match shared_ctx.shared.entry(ByAddress(ptr.weak())) {
        Entry::Occupied(entry) => *entry.get(),
        Entry::Vacant(entry) => {
            let id = shared_ctx.next_id;
            entry.insert(id);
            shared_ctx.next_id += 1;
            id
        }
    };
    <usize as Serialize<E>>::serialize(&index, e, ctx)
}

#[macro_export]
macro_rules! derive_serialize_rc_shared {
    ($ty:ty) => {
        impl<E: $crate::reexports::marshal::encode::Encoder>
            $crate::reexports::marshal::ser::rc::SerializeRc<E> for $ty
        {
            fn serialize_rc(
                this: &$crate::reexports::marshal_pointer::rc_ref::RcRef<Self>,
                e: E::AnyEncoder<'_>,
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
                e: E::AnyEncoder<'_>,
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
                e: E::AnyEncoder<'_>,
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
                e: E::AnyEncoder<'_>,
                ctx: &mut $crate::reexports::marshal::context::Context,
            ) -> anyhow::Result<()> {
                $crate::ser::serialize_arc_weak::<E, Self>(this, e, ctx)
            }
        }
    };
}
