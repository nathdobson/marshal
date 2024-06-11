use crate::context::Context;
use crate::ser::Serialize;
use marshal_core::encode::Encoder;
use std::rc::Rc;
use std::sync::Arc;
use std::{rc, sync};

impl<E: Encoder, T: ?Sized + SerializeRc<E>> Serialize<E> for Rc<T> {
    fn serialize(&self, e: E::AnyEncoder<'_>, ctx: &mut Context) -> anyhow::Result<()> {
        T::serialize_rc(self, e, ctx)
    }
}
pub trait SerializeRc<E: Encoder> {
    fn serialize_rc(this: &Rc<Self>, e: E::AnyEncoder<'_>, ctx: &mut Context)
        -> anyhow::Result<()>;
}

impl<E: Encoder, T: ?Sized + SerializeArc<E>> Serialize<E> for Arc<T> {
    fn serialize(&self, e: E::AnyEncoder<'_>, ctx: &mut Context) -> anyhow::Result<()> {
        T::serialize_arc(self, e, ctx)
    }
}
pub trait SerializeArc<E: Encoder> {
    fn serialize_arc(
        this: &Arc<Self>,
        e: E::AnyEncoder<'_>,
        ctx: &mut Context,
    ) -> anyhow::Result<()>;
}

impl<E: Encoder, T: ?Sized + SerializeRcWeak<E>> Serialize<E> for rc::Weak<T> {
    fn serialize(&self, e: E::AnyEncoder<'_>, ctx: &mut Context) -> anyhow::Result<()> {
        T::serialize_rc_weak(self, e, ctx)
    }
}
pub trait SerializeRcWeak<E: Encoder> {
    fn serialize_rc_weak(
        this: &rc::Weak<Self>,
        e: E::AnyEncoder<'_>,
        ctx: &mut Context,
    ) -> anyhow::Result<()>;
}

impl<E: Encoder, T: ?Sized + SerializeArcWeak<E>> Serialize<E> for sync::Weak<T> {
    fn serialize(&self, e: E::AnyEncoder<'_>, ctx: &mut Context) -> anyhow::Result<()> {
        T::serialize_arc_weak(self, e, ctx)
    }
}
pub trait SerializeArcWeak<E: Encoder> {
    fn serialize_arc_weak(
        this: &sync::Weak<Self>,
        e: E::AnyEncoder<'_>,
        ctx: &mut Context,
    ) -> anyhow::Result<()>;
}

#[macro_export]
macro_rules! derive_serialize_rc_transparent {
    ($ty:ty) => {
        impl<E: $crate::encode::Encoder> $crate::ser::rc::SerializeRc<E> for $ty {
            fn serialize_rc(
                this: &::std::rc::Rc<Self>,
                e: <E as $crate::encode::Encoder>::AnyEncoder<'_>,
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
                this: &::std::sync::Arc<Self>,
                e: <E as $crate::encode::Encoder>::AnyEncoder<'_>,
                ctx: &mut $crate::context::Context,
            ) -> $crate::reexports::anyhow::Result<()> {
                <Self as $crate::ser::Serialize<E>>::serialize(&**this, e, ctx)
            }
        }
    };
}
