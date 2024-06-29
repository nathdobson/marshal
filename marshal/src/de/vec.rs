use marshal_core::decode::{
    AnyDecoder, DecodeHint, DecoderView, Decoder,
};

use crate::context::Context;
use crate::de::Deserialize;

impl<D: Decoder, T: Deserialize<D>> Deserialize<D> for Vec<T> {
    default fn deserialize<'p, 'de>(
        d: AnyDecoder<'p, 'de, D>,
        mut ctx: Context,
    ) -> anyhow::Result<Self> {
        match d.decode(DecodeHint::Seq)? {
            DecoderView::Seq(mut seq) => seq
                .seq_into_iter(|x| T::deserialize(x, ctx.reborrow()))
                .collect(),
            unexpected => unexpected.mismatch("seq")?,
        }
    }
}

impl<D: Decoder> Deserialize<D> for Vec<u8> {
    fn deserialize<'p, 'de>(d: AnyDecoder<'p, 'de, D>, _ctx: Context) -> anyhow::Result<Self> {
        match d.decode(DecodeHint::Bytes)? {
            DecoderView::Bytes(x) => Ok(x.into_owned()),
            unexpected => unexpected.mismatch("bytes")?,
        }
    }
}
