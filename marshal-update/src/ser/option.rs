use marshal::context::Context;
use marshal::encode::{AnyEncoder, AnyGenEncoder, Encoder, GenEncoder};

use crate::ser::{SerializeStream, SerializeUpdate};

pub struct OptionStream<T: SerializeStream> {
    old: Option<T::Stream>,
}

impl<T: SerializeStream> SerializeStream for Option<T> {
    type Stream = OptionStream<T>;

    fn start_stream(&self, ctx: Context) -> anyhow::Result<Self::Stream> {
        if let Some(this) = self {
            Ok(OptionStream {
                old: Some(this.start_stream(ctx)?),
            })
        } else {
            Ok(OptionStream { old: None })
        }
    }
}

impl<E: GenEncoder, T: SerializeUpdate<E>> SerializeUpdate<E> for Option<T> {
    fn serialize_update<'w, 'en>(
        &self,
        stream: &mut Self::Stream,
        e: AnyGenEncoder<'w, 'en, E>,
        mut ctx: Context,
    ) -> anyhow::Result<()> {
        if let Some(new) = self {
            if let Some(old) = &mut stream.old {
                let mut e = e.encode_some()?;
                new.serialize_update(old, e.encode_some()?, ctx)?;
                e.end()?;
            } else {
                stream.old = Some(new.start_stream(ctx.reborrow())?);
                let mut e = e.encode_some()?;
                new.serialize(e.encode_some()?, ctx.reborrow())?;
                e.end()?;
            }
        } else {
            stream.old = None;
            e.encode_none()?;
        }
        Ok(())
    }
}
