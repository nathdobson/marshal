use std::collections::BTreeMap;
use marshal_core::decode::{AnyDecoder, DecodeHint, Decoder};
use marshal_core::encode::{AnyEncoder, Encoder};
use crate::context::Context;
use crate::de::Deserialize;
use crate::ser::Serialize;

impl<W: Encoder, K: Ord + Serialize<W>, V: Serialize<W>> Serialize<W> for BTreeMap<K, V> {
    fn serialize<'w, 'en>(
        &self,
        w: AnyEncoder<'w, 'en, W>,
        mut ctx: Context,
    ) -> anyhow::Result<()> {
        let mut w = w.encode_map(self.len())?;
        for (k, v) in self.iter() {
            let mut w = w.encode_entry()?;
            k.serialize(w.encode_key()?, ctx.reborrow())?;
            v.serialize(w.encode_value()?, ctx.reborrow())?;
            w.end()?;
        }
        w.end()?;
        Ok(())
    }
}

impl<D: Decoder, K: Ord + Deserialize<D>, V: Deserialize<D>> Deserialize<D> for BTreeMap<K, V> {
    fn deserialize<'p, 'de>(p: AnyDecoder<'p, 'de, D>, ctx: Context) -> anyhow::Result<Self> {
        p.decode(DecodeHint::Map)?
            .try_into_map()?
            .map_into_iter(
                ctx,
                |ctx, k| K::deserialize(k, ctx.reborrow()),
                |ctx, k, v| Ok((k, V::deserialize(v, ctx.reborrow())?)),
            )
            .collect()
    }
}