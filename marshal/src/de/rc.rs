use crate::context::Context;
use crate::de::Deserialize;
use marshal_core::decode::Decoder;
use std::sync::Arc;
use std::{rc, sync};
use std::rc::Rc;

impl<'de, D: Decoder<'de>, T: ?Sized + DeserializeArc<'de, D>> Deserialize<'de, D> for Arc<T> {
    fn deserialize<'p>(p: D::AnyDecoder<'p>, ctx: &mut Context) -> anyhow::Result<Self> {
        T::deserialize_arc(p, ctx)
    }
}

trait DeserializeArc<'de, D: Decoder<'de>> {
    fn deserialize_arc<'p>(p: D::AnyDecoder<'p>, ctx: &mut Context) -> anyhow::Result<Arc<Self>>;
}

impl<'de, D: Decoder<'de>, T: ?Sized + DeserializeRc<'de, D>> Deserialize<'de, D> for Rc<T> {
    fn deserialize<'p>(p: D::AnyDecoder<'p>, ctx: &mut Context) -> anyhow::Result<Self> {
        T::deserialize_rc(p, ctx)
    }
}

trait DeserializeRc<'de, D: Decoder<'de>> {
    fn deserialize_rc<'p>(p: D::AnyDecoder<'p>, ctx: &mut Context) -> anyhow::Result<Rc<Self>>;
}

impl<'de, D: Decoder<'de>, T: ?Sized + DeserializeArcWeak<'de, D>> Deserialize<'de, D>
    for sync::Weak<T>
{
    fn deserialize<'p>(p: D::AnyDecoder<'p>, ctx: &mut Context) -> anyhow::Result<Self> {
        T::deserialize_arc_weak(p, ctx)
    }
}

trait DeserializeArcWeak<'de, D: Decoder<'de>> {
    fn deserialize_arc_weak<'p>(
        p: D::AnyDecoder<'p>,
        ctx: &mut Context,
    ) -> anyhow::Result<sync::Weak<Self>>;
}

impl<'de, D: Decoder<'de>, T: ?Sized + DeserializeRcWeak<'de, D>> Deserialize<'de, D>
    for rc::Weak<T>
{
    fn deserialize<'p>(p: D::AnyDecoder<'p>, ctx: &mut Context) -> anyhow::Result<Self> {
        T::deserialize_rc_weak(p, ctx)
    }
}

trait DeserializeRcWeak<'de, D: Decoder<'de>> {
    fn deserialize_rc_weak<'p>(
        p: D::AnyDecoder<'p>,
        ctx: &mut Context,
    ) -> anyhow::Result<rc::Weak<Self>>;
}
