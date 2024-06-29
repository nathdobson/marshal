use marshal_core::decode::{
    AnyGenDecoder, DecodeHint, DecoderView, GenDecoder,
};

use crate::context::Context;
use crate::de::Deserialize;

impl<D: GenDecoder, T: Deserialize<D>> Deserialize<D> for Vec<T> {
    default fn deserialize<'p, 'de>(
        d: AnyGenDecoder<'p, 'de, D>,
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

impl<D: GenDecoder> Deserialize<D> for Vec<u8> {
    fn deserialize<'p, 'de>(d: AnyGenDecoder<'p, 'de, D>, _ctx: Context) -> anyhow::Result<Self> {
        match d.decode(DecodeHint::Bytes)? {
            DecoderView::Bytes(x) => Ok(x.into_owned()),
            unexpected => unexpected.mismatch("bytes")?,
        }
    }
}
