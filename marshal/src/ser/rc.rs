use std::{rc, sync};
use std::rc::Rc;
use std::sync::Arc;

use marshal_core::encode::{AnyEncoder, Encoder};
use marshal_pointer::arc_ref::ArcRef;
use marshal_pointer::arc_weak_ref::ArcWeakRef;
use marshal_pointer::AsFlatRef;
use marshal_pointer::rc_ref::RcRef;
use marshal_pointer::rc_weak_ref::RcWeakRef;

use crate::context::Context;
use crate::ser::Serialize;

impl<E: Encoder, T: ?Sized + SerializeRc<E>> Serialize<E> for Rc<T> {
    fn serialize(&self, e: AnyEncoder<'_, E>, ctx: &mut Context) -> anyhow::Result<()> {
        T::serialize_rc(self.as_flat_ref(), e, ctx)
    }
}

impl<E: Encoder, T: ?Sized + SerializeRc<E>> Serialize<E> for RcRef<T> {
    fn serialize(&self, e: AnyEncoder<'_, E>, ctx: &mut Context) -> anyhow::Result<()> {
        T::serialize_rc(self, e, ctx)
    }
}

pub trait SerializeRc<E: Encoder> {
    fn serialize_rc(
        this: &RcRef<Self>,
        e: AnyEncoder<'_, E>,
        ctx: &mut Context,
    ) -> anyhow::Result<()>;
}

impl<E: Encoder, T: ?Sized + SerializeArc<E>> Serialize<E> for Arc<T> {
    fn serialize(&self, e: AnyEncoder<'_, E>, ctx: &mut Context) -> anyhow::Result<()> {
        T::serialize_arc(self.as_flat_ref(), e, ctx)
    }
}

impl<E: Encoder, T: ?Sized + SerializeArc<E>> Serialize<E> for ArcRef<T> {
    fn serialize(&self, e: AnyEncoder<'_, E>, ctx: &mut Context) -> anyhow::Result<()> {
        T::serialize_arc(self, e, ctx)
    }
}

pub trait SerializeArc<E: Encoder> {
    fn serialize_arc(
        this: &ArcRef<Self>,
        e: AnyEncoder<'_, E>,
        ctx: &mut Context,
    ) -> anyhow::Result<()>;
}

impl<E: Encoder, T: ?Sized + SerializeRcWeak<E>> Serialize<E> for rc::Weak<T> {
    fn serialize(&self, e: AnyEncoder<'_, E>, ctx: &mut Context) -> anyhow::Result<()> {
        T::serialize_rc_weak(self.as_flat_ref(), e, ctx)
    }
}

impl<E: Encoder, T: ?Sized + SerializeRcWeak<E>> Serialize<E> for RcWeakRef<T> {
    fn serialize(&self, e: AnyEncoder<'_, E>, ctx: &mut Context) -> anyhow::Result<()> {
        T::serialize_rc_weak(self, e, ctx)
    }
}

pub trait SerializeRcWeak<E: Encoder> {
    fn serialize_rc_weak(
        this: &RcWeakRef<Self>,
        e: AnyEncoder<'_, E>,
        ctx: &mut Context,
    ) -> anyhow::Result<()>;
}

impl<E: Encoder, T: ?Sized + SerializeArcWeak<E>> Serialize<E> for sync::Weak<T> {
    fn serialize(&self, e: AnyEncoder<'_, E>, ctx: &mut Context) -> anyhow::Result<()> {
        T::serialize_arc_weak(self.as_flat_ref(), e, ctx)
    }
}

impl<E: Encoder, T: ?Sized + SerializeArcWeak<E>> Serialize<E> for ArcWeakRef<T> {
    fn serialize(&self, e: AnyEncoder<'_, E>, ctx: &mut Context) -> anyhow::Result<()> {
        T::serialize_arc_weak(self, e, ctx)
    }
}

pub trait SerializeArcWeak<E: Encoder> {
    fn serialize_arc_weak(
        this: &ArcWeakRef<Self>,
        e: AnyEncoder<'_, E>,
        ctx: &mut Context,
    ) -> anyhow::Result<()>;
}

#[macro_export]
macro_rules! derive_serialize_rc_transparent {
    ($ty:ty) => {
        impl<E: $crate::encode::Encoder> $crate::ser::rc::SerializeRc<E> for $ty {
            fn serialize_rc(
                this: &$crate::reexports::marshal_pointer::rc_ref::RcRef<Self>,
                e: $crate::encode::AnyEncoder<'_, E>,
                ctx: &mut $crate::context::Context,
            ) -> $crate::reexports::anyhow::Result<()> {
                <Self as $crate::ser::Serialize<E>>::serialize(&**this, e, ctx)
            }
        }
    };
}

#[macro_export]
macro_rules! derive_serialize_arc_transparent {
    ($ty:ty) => {
        impl<E: $crate::encode::Encoder> $crate::ser::rc::SerializeArc<E> for $ty {
            fn serialize_arc(
                this: &$crate::reexports::marshal_pointer::arc_ref::ArcRef<Self>,
                e: $crate::encode::AnyEncoder<'_, E>,
                ctx: &mut $crate::context::Context,
            ) -> $crate::reexports::anyhow::Result<()> {
                <Self as $crate::ser::Serialize<E>>::serialize(&**this, e, ctx)
            }
        }
    };
}
