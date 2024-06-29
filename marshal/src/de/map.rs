use std::collections::{BTreeMap, HashMap};
use std::hash::Hash;

use marshal_core::decode::{AnyDecoder, DecodeHint, Decoder};

use crate::context::Context;
use crate::de::Deserialize;

impl<P: Decoder, K: Hash + Eq + Deserialize<P>, V: Deserialize<P>> Deserialize<P>
    for HashMap<K, V>
{
    fn deserialize<'p>(p: AnyDecoder<'p, P>, mut ctx: Context) -> anyhow::Result<Self> {
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

impl<P: Decoder, K: Ord + Deserialize<P>, V: Deserialize<P>> Deserialize<P> for BTreeMap<K, V> {
    fn deserialize<'p>(p: AnyDecoder<'p, P>, ctx: Context) -> anyhow::Result<Self> {
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
