use marshal_core::decode::{AnyDecoder, DecodeHint, DecoderView, Decoder};

use crate::context::Context;
use crate::de::Deserialize;

impl<D:Decoder, T: Deserialize<D>> Deserialize<D> for Option<T> {
    fn deserialize<'p,'de>(p: AnyDecoder<'p, 'de, D>, ctx: Context) -> anyhow::Result<Self> {
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
