use std::any::{type_name, Any};

use marshal::context::Context;
use marshal::encode::{AnyEncoder, Encoder};
use marshal::ser::Serialize;
use marshal_pointer::RawAny;

mod boxed;
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
    fn start_stream_dyn(&self, ctx: Context) -> anyhow::Result<Box<dyn Sync + Send + RawAny>>;
}

pub trait SerializeUpdateDyn<E: Encoder>: 'static + Serialize<E> + SerializeStreamDyn {
    fn serialize_update_dyn<'w, 'en>(
        &self,
        stream: &mut Box<dyn Sync + Send + RawAny>,
        e: AnyEncoder<'w, 'en, E>,
        ctx: Context,
    ) -> anyhow::Result<()>;
}

impl<T: SerializeStream<Stream: 'static> + Any> SerializeStreamDyn for T {
    fn start_stream_dyn(&self, ctx: Context) -> anyhow::Result<Box<dyn Sync + Send + RawAny>> {
        Ok(Box::new(self.start_stream(ctx)?))
    }
}

impl<E: Encoder, T: 'static + SerializeUpdate<E> + SerializeStream<Stream: 'static>>
    SerializeUpdateDyn<E> for T
{
    fn serialize_update_dyn<'w, 'en>(
        &self,
        stream: &mut Box<dyn Sync + Send + RawAny>,
        e: AnyEncoder<'w, 'en, E>,
        ctx: Context,
    ) -> anyhow::Result<()> {
        let stream = (&mut **stream) as &mut dyn RawAny;
        let name = (stream as *const dyn RawAny).raw_type_name();
        Ok(self.serialize_update(
            (stream as &mut dyn Any)
                .downcast_mut::<T::Stream>()
                .unwrap_or_else(
                    || panic!("expected {}, found {}", type_name::<T::Stream>(), name,),
                ),
            e,
            ctx,
        )?)
    }
}
