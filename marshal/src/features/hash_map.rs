use std::collections::HashMap;
use std::hash::Hash;

use marshal_core::decode::{AnyDecoder, DecodeHint, Decoder};

use crate::context::Context;
use crate::de::Deserialize;
use marshal_core::encode::{AnyEncoder, Encoder};

use crate::ser::Serialize;

impl<D: Decoder, K: Hash + Eq + Deserialize<D>, V: Deserialize<D>> Deserialize<D>
    for HashMap<K, V>
{
    fn deserialize<'p, 'de>(p: AnyDecoder<'p, 'de, D>, mut ctx: Context) -> anyhow::Result<Self> {
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

impl<W: Encoder, K: Eq + Hash + Serialize<W>, V: Serialize<W>> Serialize<W> for HashMap<K, V> {
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
