use std::collections::{BTreeMap, HashMap};
use std::hash::Hash;

use marshal_core::decode::{AnyGenDecoder, DecodeHint, GenDecoder};

use crate::context::Context;
use crate::de::Deserialize;

impl<D: GenDecoder, K: Hash + Eq + Deserialize<D>, V: Deserialize<D>> Deserialize<D>
    for HashMap<K, V>
{
    fn deserialize<'p, 'de>(
        p: AnyGenDecoder<'p, 'de, D>,
        mut ctx: Context,
    ) -> anyhow::Result<Self> {
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

impl<D: GenDecoder, K: Ord + Deserialize<D>, V: Deserialize<D>> Deserialize<D> for BTreeMap<K, V> {
    fn deserialize<'p, 'de>(p: AnyGenDecoder<'p, 'de, D>, ctx: Context) -> anyhow::Result<Self> {
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
