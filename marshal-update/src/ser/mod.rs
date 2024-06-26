use std::any::Any;

use marshal::context::Context;
use marshal::decode::{AnyDecoder, Decoder};
use marshal::encode::{AnyEncoder, Encoder};
use marshal::ser::Serialize;

use crate::de::DeserializeUpdate;

mod derive_serialize_update_for_clone;
mod option;
mod rc;
pub mod set_channel;
mod tuple;

pub trait SerializeStream {
    type Stream: Sync + Send;
    fn start_stream(&self, ctx: Context) -> anyhow::Result<Self::Stream>;
}

pub trait SerializeUpdate<E: Encoder>: Serialize<E> + SerializeStream {
    fn serialize_update(
        &self,
        stream: &mut Self::Stream,
        e: AnyEncoder<E>,
        ctx: Context,
    ) -> anyhow::Result<()>;
}

pub trait SerializeStreamDyn: Any {
    fn start_stream_dyn(&self, ctx: Context) -> anyhow::Result<Box<dyn Sync + Send + Any>>;
}

pub trait SerializeUpdateDyn<E: Encoder>: 'static + Serialize<E> + SerializeStreamDyn {
    fn serialize_update_dyn<'w, 'en>(
        &self,
        stream: &mut Box<dyn Sync + Send + Any>,
        e: AnyEncoder<'w, 'en, E>,
        ctx: Context,
    ) -> anyhow::Result<()>;
}

impl<T: SerializeStream<Stream: 'static> + Any> SerializeStreamDyn for T {
    fn start_stream_dyn(&self, ctx: Context) -> anyhow::Result<Box<dyn Sync + Send + Any>> {
        Ok(Box::new(self.start_stream(ctx)?))
    }
}

impl<E: Encoder, T: 'static + SerializeUpdate<E> + SerializeStream<Stream: 'static>>
    SerializeUpdateDyn<E> for T
{
    fn serialize_update_dyn<'w, 'en>(
        &self,
        stream: &mut Box<dyn Sync + Send + Any>,
        e: AnyEncoder<'w, 'en, E>,
        ctx: Context,
    ) -> anyhow::Result<()> {
        Ok(self.serialize_update((**stream).downcast_mut().unwrap(), e, ctx)?)
    }
}

pub trait DeserializeUpdateDyn<D: Decoder>: 'static + Any {
    fn deserialize_update_dyn<'p, 'de>(
        &mut self,
        d: AnyDecoder<'p, 'de, D>,
        ctx: Context,
    ) -> anyhow::Result<()>;
}

impl<D: Decoder, T: 'static + Any + DeserializeUpdate<D>> DeserializeUpdateDyn<D> for T {
    fn deserialize_update_dyn<'p, 'de>(
        &mut self,
        d: AnyDecoder<'p, 'de, D>,
        ctx: Context,
    ) -> anyhow::Result<()> {
        self.deserialize_update(d, ctx)
    }
}
