use marshal_core::decode::{
    AnyDecoder, DecodeHint, Decoder, DecoderView,
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

use marshal_core::encode::{AnyEncoder, Encoder};

use crate::ser::Serialize;

impl<W: Encoder, T: Serialize<W>> Serialize<W> for Vec<T> {
    default fn serialize<'w, 'en>(
        &self,
        w: AnyEncoder<'w, 'en, W>,
        mut ctx: Context,
    ) -> anyhow::Result<()> {
        let mut w = w.encode_seq(self.len())?;
        for x in self.iter() {
            x.serialize(w.encode_element()?, ctx.reborrow())?;
        }
        w.end()?;
        Ok(())
    }
}

impl<W: Encoder> Serialize<W> for Vec<u8> {
    fn serialize<'w, 'en>(
        &self,
        w: AnyEncoder<'w, 'en, W>,
        _ctx: Context,
    ) -> anyhow::Result<()> {
        w.encode_bytes(self)
    }
}
