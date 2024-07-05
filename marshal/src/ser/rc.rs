use marshal_core::encode::{AnyEncoder, Encoder};
use marshal_pointer::{Arcf, ArcfRef, ArcfWeak, ArcfWeakRef, AsFlatRef, Rcf, RcfRef, RcfWeak, RcfWeakRef};

use crate::context::Context;
use crate::ser::Serialize;

impl<E: Encoder, T: ?Sized + SerializeRc<E>> Serialize<E> for Rcf<T> {
    fn serialize<'w, 'en>(&self, e: AnyEncoder<'w, 'en, E>, ctx: Context) -> anyhow::Result<()> {
        T::serialize_rc(self.as_flat_ref(), e, ctx)
    }
}

impl<E: Encoder, T: ?Sized + SerializeRc<E>> Serialize<E> for RcfRef<T> {
    fn serialize<'w, 'en>(&self, e: AnyEncoder<'w, 'en, E>, ctx: Context) -> anyhow::Result<()> {
        T::serialize_rc(self, e, ctx)
    }
}

pub trait SerializeRc<E: Encoder> {
    fn serialize_rc<'w, 'en>(
        this: &RcfRef<Self>,
        e: AnyEncoder<'w, 'en, E>,
        ctx: Context,
    ) -> anyhow::Result<()>;
}

impl<E: Encoder, T: ?Sized + SerializeArc<E>> Serialize<E> for Arcf<T> {
    fn serialize<'w, 'en>(&self, e: AnyEncoder<'w, 'en, E>, ctx: Context) -> anyhow::Result<()> {
        T::serialize_arc(self.as_flat_ref(), e, ctx)
    }
}

impl<E: Encoder, T: ?Sized + SerializeArc<E>> Serialize<E> for ArcfRef<T> {
    fn serialize<'w, 'en>(&self, e: AnyEncoder<'w, 'en, E>, ctx: Context) -> anyhow::Result<()> {
        T::serialize_arc(self, e, ctx)
    }
}

pub trait SerializeArc<E: Encoder> {
    fn serialize_arc<'w, 'en>(
        this: &ArcfRef<Self>,
        e: AnyEncoder<'w, 'en, E>,
        ctx: Context,
    ) -> anyhow::Result<()>;
}

impl<E: Encoder, T: ?Sized + SerializeRcWeak<E>> Serialize<E> for RcfWeak<T> {
    fn serialize<'w, 'en>(&self, e: AnyEncoder<'w, 'en, E>, ctx: Context) -> anyhow::Result<()> {
        T::serialize_rc_weak(self.as_flat_ref(), e, ctx)
    }
}

impl<E: Encoder, T: ?Sized + SerializeRcWeak<E>> Serialize<E> for RcfWeakRef<T> {
    fn serialize<'w, 'en>(&self, e: AnyEncoder<'w, 'en, E>, ctx: Context) -> anyhow::Result<()> {
        T::serialize_rc_weak(self, e, ctx)
    }
}

pub trait SerializeRcWeak<E: Encoder> {
    fn serialize_rc_weak<'w, 'en>(
        this: &RcfWeakRef<Self>,
        e: AnyEncoder<'w, 'en, E>,
        ctx: Context,
    ) -> anyhow::Result<()>;
}

impl<E: Encoder, T: ?Sized + SerializeArcWeak<E>> Serialize<E> for ArcfWeak<T> {
    fn serialize<'w, 'en>(&self, e: AnyEncoder<'w, 'en, E>, ctx: Context) -> anyhow::Result<()> {
        T::serialize_arc_weak(self.as_flat_ref(), e, ctx)
    }
}

impl<E: Encoder, T: ?Sized + SerializeArcWeak<E>> Serialize<E> for ArcfWeakRef<T> {
    fn serialize<'w, 'en>(&self, e: AnyEncoder<'w, 'en, E>, ctx: Context) -> anyhow::Result<()> {
        T::serialize_arc_weak(self, e, ctx)
    }
}

pub trait SerializeArcWeak<E: Encoder> {
    fn serialize_arc_weak<'w, 'en>(
        this: &ArcfWeakRef<Self>,
        e: AnyEncoder<'w, 'en, E>,
        ctx: Context,
    ) -> anyhow::Result<()>;
}

#[macro_export]
macro_rules! derive_serialize_rc_transparent {
    ($ty:ty) => {
        impl<E: $crate::encode::Encoder> $crate::ser::rc::SerializeRc<E> for $ty {
            fn serialize_rc<'w, 'en>(
                this: &$crate::reexports::marshal_pointer::RcfRef<Self>,
                e: $crate::encode::AnyEncoder<'w, 'en, E>,
                mut ctx: $crate::context::Context,
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
            fn serialize_arc<'w, 'en>(
                this: &$crate::reexports::marshal_pointer::ArcfRef<Self>,
                e: $crate::encode::AnyEncoder<'w, 'en, E>,
                mut ctx: $crate::context::Context,
            ) -> $crate::reexports::anyhow::Result<()> {
                <Self as $crate::ser::Serialize<E>>::serialize(&**this, e, ctx)
            }
        }
    };
}
