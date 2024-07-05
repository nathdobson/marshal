use std::rc::Rc;
use std::sync::Arc;
use std::{rc, sync};

use crate::context::Context;
use crate::de::Deserialize;
use marshal_core::decode::{AnyDecoder, Decoder};

impl<D: Decoder, T: ?Sized + DeserializeArc<D>> Deserialize<D> for Arc<T> {
    fn deserialize<'p, 'de>(d: AnyDecoder<'p, 'de, D>, ctx: Context) -> anyhow::Result<Self> {
        T::deserialize_arc(d, ctx)
    }
}

pub trait DeserializeArc<D: Decoder> {
    fn deserialize_arc<'p, 'de>(
        p: AnyDecoder<'p, 'de, D>,
        ctx: Context,
    ) -> anyhow::Result<Arc<Self>>;
}

impl<D: Decoder, T: ?Sized + DeserializeRc<D>> Deserialize<D> for Rc<T> {
    fn deserialize<'p, 'de>(p: AnyDecoder<'p, 'de, D>, ctx: Context) -> anyhow::Result<Self> {
        T::deserialize_rc(p, ctx)
    }
}

impl<D: Decoder, T: ?Sized + DeserializeRc<D>> Deserialize<D> for Rcf<T> {
    fn deserialize<'p, 'de>(p: AnyDecoder<'p, 'de, D>, ctx: Context) -> anyhow::Result<Self> {
        Ok(T::deserialize_rc(p, ctx)?.into())
    }
}

pub trait DeserializeRc<D: Decoder> {
    fn deserialize_rc<'p, 'de>(p: AnyDecoder<'p, 'de, D>, ctx: Context)
        -> anyhow::Result<Rc<Self>>;
}

impl<D: Decoder, T: ?Sized + DeserializeArcWeak<D>> Deserialize<D> for sync::Weak<T> {
    fn deserialize<'p, 'de>(p: AnyDecoder<'p, 'de, D>, ctx: Context) -> anyhow::Result<Self> {
        T::deserialize_arc_weak(p, ctx)
    }
}

pub trait DeserializeArcWeak<D: Decoder> {
    fn deserialize_arc_weak<'p, 'de>(
        d: AnyDecoder<'p, 'de, D>,
        ctx: Context,
    ) -> anyhow::Result<sync::Weak<Self>>;
}

impl<D: Decoder, T: ?Sized + DeserializeRcWeak<D>> Deserialize<D> for rc::Weak<T> {
    fn deserialize<'p, 'de>(p: AnyDecoder<'p, 'de, D>, ctx: Context) -> anyhow::Result<Self> {
        T::deserialize_rc_weak(p, ctx)
    }
}

impl<D: Decoder, T: ?Sized + DeserializeRcWeak<D>> Deserialize<D> for RcfWeak<T> {
    fn deserialize<'p, 'de>(p: AnyDecoder<'p, 'de, D>, ctx: Context) -> anyhow::Result<Self> {
        Ok(T::deserialize_rc_weak(p, ctx)?.into())
    }
}

pub trait DeserializeRcWeak<D: Decoder> {
    fn deserialize_rc_weak<'p, 'de>(
        p: AnyDecoder<'p, 'de, D>,
        ctx: Context,
    ) -> anyhow::Result<rc::Weak<Self>>;
}

#[macro_export]
macro_rules! derive_deserialize_rc_transparent {
    ($ty:ty) => {
        impl<D: $crate::decode::Decoder> $crate::de::rc::DeserializeRc<D> for $ty {
            fn deserialize_rc<'p, 'de>(
                p: $crate::decode::AnyDecoder<'p, 'de, D>,
                mut ctx: $crate::context::Context,
            ) -> $crate::reexports::anyhow::Result<::std::rc::Rc<Self>> {
                ::std::result::Result::Ok(::std::rc::Rc::new(<$ty as $crate::de::Deserialize<
                    D,
                >>::deserialize(p, ctx)?))
            }
        }
    };
}

#[macro_export]
macro_rules! derive_deserialize_arc_transparent {
    ($ty:ty) => {
        impl<D: $crate::decode::Decoder> $crate::de::rc::DeserializeArc<D> for $ty {
            fn deserialize_arc<'p, 'de>(
                p: $crate::decode::AnyDecoder<'p, 'de, D>,
                mut ctx: $crate::context::Context,
            ) -> $crate::reexports::anyhow::Result<::std::sync::Arc<Self>> {
                ::std::result::Result::Ok(::std::sync::Arc::new(
                    <$ty as $crate::de::Deserialize<D>>::deserialize(p, ctx)?,
                ))
            }
        }
    };
}
