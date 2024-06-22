use marshal::context::Context;
use marshal::encode::{AnyEncoder, Encoder};

use crate::ser::{SerializeStream, SerializeUpdate};

impl<T1: SerializeStream, T2: SerializeStream> SerializeStream for (T1, T2) {
    type Stream = (T1::Stream, T2::Stream);
    fn start_stream(&self, ctx: &mut Context) -> anyhow::Result<Self::Stream> {
        Ok((self.0.start_stream(ctx)?, self.1.start_stream(ctx)?))
    }
}

impl<T1: SerializeUpdate<E>, T2: SerializeUpdate<E>, E: Encoder> SerializeUpdate<E> for (T1, T2) {
    fn serialize_update(
        &self,
        stream: &mut Self::Stream,
        e: AnyEncoder<E>,
        ctx: &mut Context,
    ) -> anyhow::Result<()> {
        let mut e = e.encode_tuple(2)?;
        self.0
            .serialize_update(&mut stream.0, e.encode_element()?, ctx)?;
        self.1
            .serialize_update(&mut stream.1, e.encode_element()?, ctx)?;
        e.end()?;
        Ok(())
    }
}
