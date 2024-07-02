use crate::ser::{SerializeStream, SerializeUpdate};
use marshal::context::Context;
use marshal::encode::{AnyEncoder, Encoder};

pub struct BoxedStream<T>(T);

impl<T: SerializeStream> SerializeStream for Box<T> {
    type Stream = BoxedStream<T::Stream>;

    fn start_stream(&self, ctx: Context) -> anyhow::Result<Self::Stream> {
        Ok(BoxedStream((**self).start_stream(ctx)?))
    }
}

impl<E: Encoder, T: SerializeUpdate<E>> SerializeUpdate<E> for Box<T> {
    fn serialize_update(
        &self,
        stream: &mut Self::Stream,
        e: AnyEncoder<E>,
        ctx: Context,
    ) -> anyhow::Result<()> {
        (**self).serialize_update(&mut stream.0, e, ctx)
    }
}
