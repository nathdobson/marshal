use marshal::context::Context;
use marshal::encode::{AnyEncoder, Encoder};

use crate::ser::{SerializeStream, SerializeUpdate};

impl<T1: SerializeStream, T2: SerializeStream> SerializeStream for (T1, T2) {
    type Stream = (T1::Stream, T2::Stream);
    fn start_stream(&self, mut ctx: Context) -> anyhow::Result<Self::Stream> {
        Ok((
            self.0.start_stream(ctx.reborrow())?,
            self.1.start_stream(ctx.reborrow())?,
        ))
    }
}

impl<E: Encoder, T1: SerializeUpdate<E>, T2: SerializeUpdate<E>> SerializeUpdate<E>
    for (T1, T2)
{
    fn serialize_update<'w, 'en>(
        &self,
        stream: &mut Self::Stream,
        e: AnyEncoder<'w, 'en, E>,
        mut ctx: Context,
    ) -> anyhow::Result<()> {
        let mut e = e.encode_tuple(2)?;
        self.0
            .serialize_update(&mut stream.0, e.encode_element()?, ctx.reborrow())?;
        self.1
            .serialize_update(&mut stream.1, e.encode_element()?, ctx.reborrow())?;
        e.end()?;
        Ok(())
    }
}
