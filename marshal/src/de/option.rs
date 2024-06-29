use marshal_core::decode::{AnyDecoder, DecodeHint, Decoder, DecoderView};

use crate::context::Context;
use crate::de::Deserialize;

impl<P: Decoder, T: Deserialize<P>> Deserialize<P> for Option<T> {
    fn deserialize<'p>(p: AnyDecoder<'p, P>, ctx: Context) -> anyhow::Result<Self> {
        match p.decode(DecodeHint::Option)? {
            DecoderView::None => Ok(None),
            DecoderView::Some(mut p) => {
                let result = Some(T::deserialize(p.decode_some()?, ctx)?);
                p.decode_end()?;
                Ok(result)
            }
            x => x.mismatch("option")?,
        }
    }
}
