use marshal_core::decode::{AnyDecoder, DecodeHint, Decoder, DecoderView};

use crate::context::Context;
use crate::de::Deserialize;
use marshal_core::encode::{AnyEncoder, Encoder};

use crate::ser::Serialize;

impl<D: Decoder, T: Deserialize<D>> Deserialize<D> for Option<T> {
    fn deserialize<'p, 'de>(p: AnyDecoder<'p, 'de, D>, ctx: Context) -> anyhow::Result<Self> {
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

impl<W: Encoder, T: Serialize<W>> Serialize<W> for Option<T> {
    fn serialize<'w, 'en>(&self, w: AnyEncoder<'w, 'en, W>, ctx: Context) -> anyhow::Result<()> {
        match self {
            None => w.encode_none(),
            Some(x) => {
                let mut w = w.encode_some()?;
                x.serialize(w.encode_some()?, ctx)?;
                w.end()?;
                Ok(())
            }
        }
    }
}
