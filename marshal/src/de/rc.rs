use std::{rc, sync};
use std::rc::Rc;
use std::sync::Arc;

use marshal_core::decode::{AnyDecoder, Decoder};

use crate::context::Context;
use crate::de::Deserialize;

impl<'de, D: Decoder<'de>, T: ?Sized + DeserializeArc<'de, D>> Deserialize<'de, D> for Arc<T> {
    fn deserialize<'p>(d: AnyDecoder<'p, 'de, D>, ctx: &mut Context) -> anyhow::Result<Self> {
        T::deserialize_arc(d, ctx)
    }
}

pub trait DeserializeArc<'de, D: Decoder<'de>> {
    fn deserialize_arc<'p>(
        p: AnyDecoder<'p, 'de, D>,
        ctx: &mut Context,
    ) -> anyhow::Result<Arc<Self>>;
}

impl<'de, D: Decoder<'de>, T: ?Sized + DeserializeRc<'de, D>> Deserialize<'de, D> for Rc<T> {
    fn deserialize<'p>(p: AnyDecoder<'p, 'de, D>, ctx: &mut Context) -> anyhow::Result<Self> {
        T::deserialize_rc(p, ctx)
    }
}

pub trait DeserializeRc<'de, D: Decoder<'de>> {
    fn deserialize_rc<'p>(p: AnyDecoder<'p, 'de, D>, ctx: &mut Context)
        -> anyhow::Result<Rc<Self>>;
}

impl<'de, D: Decoder<'de>, T: ?Sized + DeserializeArcWeak<'de, D>> Deserialize<'de, D>
    for sync::Weak<T>
{
    fn deserialize<'p>(p: AnyDecoder<'p, 'de, D>, ctx: &mut Context) -> anyhow::Result<Self> {
        T::deserialize_arc_weak(p, ctx)
    }
}

pub trait DeserializeArcWeak<'de, D: Decoder<'de>> {
    fn deserialize_arc_weak<'p>(
        p: AnyDecoder<'p, 'de, D>,
        ctx: &mut Context,
    ) -> anyhow::Result<sync::Weak<Self>>;
}

impl<'de, D: Decoder<'de>, T: ?Sized + DeserializeRcWeak<'de, D>> Deserialize<'de, D>
    for rc::Weak<T>
{
    fn deserialize<'p>(p: AnyDecoder<'p, 'de, D>, ctx: &mut Context) -> anyhow::Result<Self> {
        T::deserialize_rc_weak(p, ctx)
    }
}

pub trait DeserializeRcWeak<'de, D: Decoder<'de>> {
    fn deserialize_rc_weak<'p>(
        p: AnyDecoder<'p, 'de, D>,
        ctx: &mut Context,
    ) -> anyhow::Result<rc::Weak<Self>>;
}

#[macro_export]
macro_rules! derive_deserialize_rc_transparent {
    ($ty:ty) => {
        impl<'de, D: $crate::decode::Decoder<'de>> $crate::de::rc::DeserializeRc<'de, D> for $ty {
            fn deserialize_rc<'p>(
                p: $crate::decode::AnyDecoder<'p, 'de, D>,
                ctx: &mut $crate::context::Context,
            ) -> $crate::reexports::anyhow::Result<::std::rc::Rc<Self>> {
                ::std::result::Result::Ok(::std::rc::Rc::new(<$ty as $crate::de::Deserialize<
                    'de,
                    D,
                >>::deserialize(p, ctx)?))
            }
        }
    };
}

#[macro_export]
macro_rules! derive_deserialize_arc_transparent {
    ($ty:ty) => {
        impl<'de, D: $crate::decode::Decoder<'de>> $crate::de::rc::DeserializeArc<'de, D> for $ty {
            fn deserialize_arc<'p>(
                p: $crate::decode::AnyDecoder<'p,'de,D>,
                ctx: &mut $crate::context::Context,
            ) -> $crate::reexports::anyhow::Result<::std::sync::Arc<Self>> {
                ::std::result::Result::Ok(::std::sync::Arc::new(
                    <$ty as $crate::de::Deserialize<'de, D>>::deserialize(p, ctx)?,
                ))
            }
        }
    };
}
