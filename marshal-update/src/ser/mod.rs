use std::any::Any;

use marshal::context::Context;
use marshal::encode::{AnyEncoder, Encoder};
use marshal::ser::Serialize;

mod derive_serialize_update_for_clone;
mod rc;

pub trait SerializeStream {
    type Stream;
    fn start_stream(&self, ctx: &mut Context) -> anyhow::Result<Self::Stream>;
}

pub trait SerializeUpdate<E: Encoder>: Serialize<E> + SerializeStream {
    fn serialize_update(
        &self,
        stream: &mut Self::Stream,
        e: AnyEncoder<E>,
        ctx: &mut Context,
    ) -> anyhow::Result<()>;
}

pub trait SerializeUpdateDyn<E: Encoder>: 'static + Serialize<E> {
    fn start_stream_dyn(&self, ctx: &mut Context) -> anyhow::Result<Box<dyn Any>>;
    fn serialize_update_dyn(
        &self,
        stream: &mut dyn Any,
        e: AnyEncoder<E>,
        ctx: &mut Context,
    ) -> anyhow::Result<()>;
}

impl<E: Encoder, T: 'static + SerializeUpdate<E> + SerializeStream<Stream: 'static>>
    SerializeUpdateDyn<E> for T
{
    fn start_stream_dyn(&self, ctx: &mut Context) -> anyhow::Result<Box<dyn Any>> {
        Ok(Box::new(self.start_stream(ctx)?))
    }

    fn serialize_update_dyn(
        &self,
        stream: &mut dyn Any,
        e: AnyEncoder<E>,
        ctx: &mut Context,
    ) -> anyhow::Result<()> {
        Ok(self.serialize_update(stream.downcast_mut().unwrap(), e, ctx)?)
    }
}
