use marshal_core::decode::{AnyDecoder, DecodeHint, Decoder, DecoderView};

use crate::context::Context;
use crate::de::Deserialize;

impl<'de, P: Decoder<'de>, T: Deserialize<'de, P>> Deserialize<'de, P> for Vec<T> {
    default fn deserialize<'p>(
        p: AnyDecoder<'p, 'de, P>,
        mut ctx: Context,
    ) -> anyhow::Result<Self> {
        match p.decode(DecodeHint::Seq)? {
            DecoderView::Seq(mut seq) => seq.seq_into_iter(|x| T::deserialize(x, ctx.reborrow())).collect(),
            unexpected => unexpected.mismatch("seq")?,
        }
    }
}

impl<'de, P: Decoder<'de>> Deserialize<'de, P> for Vec<u8> {
    fn deserialize<'p>(p: AnyDecoder<'p, 'de, P>, _ctx: Context) -> anyhow::Result<Self> {
        match p.decode(DecodeHint::Bytes)? {
            DecoderView::Bytes(x) => Ok(x.into_owned()),
            unexpected => unexpected.mismatch("bytes")?,
        }
    }
}
