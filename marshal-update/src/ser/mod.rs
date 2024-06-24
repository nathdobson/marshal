use std::any::Any;

use marshal::context::Context;
use marshal::decode::{AnyDecoder, Decoder};
use marshal::encode::{AnyEncoder, Encoder};
use marshal::ser::Serialize;

use crate::de::DeserializeUpdate;

mod derive_serialize_update_for_clone;
mod option;
mod rc;
mod tuple;
pub mod set_channel;

pub trait SerializeStream {
    type Stream: Sync + Send;
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
    fn start_stream_dyn(&self, ctx: &mut Context) -> anyhow::Result<Box<dyn Sync + Send + Any>>;
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
    fn start_stream_dyn(&self, ctx: &mut Context) -> anyhow::Result<Box<dyn Sync + Send + Any>> {
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

pub trait DeserializeUpdateDyn<'de, D: Decoder<'de>>: 'static {
    fn deserialize_update_dyn(
        &mut self,
        d: AnyDecoder<'_, 'de, D>,
        ctx: &mut Context,
    ) -> anyhow::Result<()>;
}

impl<'de, D: Decoder<'de>, T: 'static + DeserializeUpdate<'de, D>> DeserializeUpdateDyn<'de, D>
    for T
{
    fn deserialize_update_dyn(
        &mut self,
        d: AnyDecoder<'_, 'de, D>,
        ctx: &mut Context,
    ) -> anyhow::Result<()> {
        self.deserialize_update(d, ctx)
    }
}
