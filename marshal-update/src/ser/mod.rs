use std::any::Any;

use marshal::context::Context;
use marshal::decode::{AnyGenDecoder, GenDecoder};
use marshal::encode::{AnyGenEncoder, GenEncoder};
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

pub trait SerializeUpdate<E: GenEncoder>: Serialize<E> + SerializeStream {
    fn serialize_update(
        &self,
        stream: &mut Self::Stream,
        e: AnyGenEncoder<E>,
        ctx: Context,
    ) -> anyhow::Result<()>;
}

pub trait SerializeStreamDyn: Any {
    fn start_stream_dyn(&self, ctx: Context) -> anyhow::Result<Box<dyn Sync + Send + Any>>;
}

pub trait SerializeUpdateDyn<E: GenEncoder>: 'static + Serialize<E> + SerializeStreamDyn {
    fn serialize_update_dyn<'w, 'en>(
        &self,
        stream: &mut Box<dyn Sync + Send + Any>,
        e: AnyGenEncoder<'w, 'en, E>,
        ctx: Context,
    ) -> anyhow::Result<()>;
}

impl<T: SerializeStream<Stream: 'static> + Any> SerializeStreamDyn for T {
    fn start_stream_dyn(&self, ctx: Context) -> anyhow::Result<Box<dyn Sync + Send + Any>> {
        Ok(Box::new(self.start_stream(ctx)?))
    }
}

impl<E: GenEncoder, T: 'static + SerializeUpdate<E> + SerializeStream<Stream: 'static>>
    SerializeUpdateDyn<E> for T
{
    fn serialize_update_dyn<'w, 'en>(
        &self,
        stream: &mut Box<dyn Sync + Send + Any>,
        e: AnyGenEncoder<'w, 'en, E>,
        ctx: Context,
    ) -> anyhow::Result<()> {
        Ok(self.serialize_update((**stream).downcast_mut().unwrap(), e, ctx)?)
    }
}

pub trait DeserializeUpdateDyn<D: GenDecoder>: 'static + Any {
    fn deserialize_update_dyn<'p, 'de>(
        &mut self,
        d: AnyGenDecoder<'p, 'de, D>,
        ctx: Context,
    ) -> anyhow::Result<()>;
}

impl<D: GenDecoder, T: 'static + Any + DeserializeUpdate<D>> DeserializeUpdateDyn<D> for T {
    fn deserialize_update_dyn<'p, 'de>(
        &mut self,
        d: AnyGenDecoder<'p, 'de, D>,
        ctx: Context,
    ) -> anyhow::Result<()> {
        self.deserialize_update(d, ctx)
    }
}
