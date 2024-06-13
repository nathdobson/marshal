use marshal::context::Context;
use marshal::encode::Encoder;
use marshal::reexports::anyhow;
use marshal::ser::Serialize;
use marshal::Serialize;
use marshal_pointer::rc_ref::RcRef;
use std::any::Any;
use std::{rc, sync};
use std::rc::Rc;
use weak_table::ptr_weak_key_hash_map::Entry;
use weak_table::PtrWeakKeyHashMap;
use marshal_pointer::arc_ref::ArcRef;

#[derive(Default)]
pub struct SharedSerializeContext {
    next_id: usize,
    rcs: PtrWeakKeyHashMap<rc::Weak<dyn Any>, usize>,
    arcs: PtrWeakKeyHashMap<sync::Weak<dyn Any>, usize>,
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
    let shared_ctx = ctx.get_or_default::<SharedSerializeContext>();
    match shared_ctx.rcs.entry(ptr.rc()) {
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

pub fn serialize_arc<E: Encoder, T: 'static + Serialize<E>>(
    ptr: &ArcRef<T>,
    e: E::AnyEncoder<'_>,
    ctx: &mut Context,
) -> anyhow::Result<()> {
    let shared_ctx = ctx.get_or_default::<SharedSerializeContext>();
    match shared_ctx.arcs.entry(ptr.arc()) {
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
