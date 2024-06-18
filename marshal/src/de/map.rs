use std::collections::{BTreeMap, HashMap};
use std::hash::Hash;

use marshal_core::decode::{AnyDecoder, DecodeHint, Decoder};

use crate::context::Context;
use crate::de::Deserialize;

impl<'de, P: Decoder<'de>, K: Hash + Eq + Deserialize<'de, P>, V: Deserialize<'de, P>>
    Deserialize<'de, P> for HashMap<K, V>
{
    fn deserialize<'p>(p: AnyDecoder<'p, 'de, P>, ctx: &mut Context) -> anyhow::Result<Self> {
        p.decode(DecodeHint::Map)?
            .try_into_map()?
            .map_into_iter(
                ctx,
                |ctx, k| K::deserialize(k, ctx),
                |ctx, k, v| Ok((k, V::deserialize(v, ctx)?)),
            )
            .collect()
    }
}

impl<'de, P: Decoder<'de>, K: Ord + Deserialize<'de, P>, V: Deserialize<'de, P>> Deserialize<'de, P>
    for BTreeMap<K, V>
{
    fn deserialize<'p>(p: AnyDecoder<'p, 'de, P>, ctx: &mut Context) -> anyhow::Result<Self> {
        p.decode(DecodeHint::Map)?
            .try_into_map()?
            .map_into_iter(
                ctx,
                |ctx, k| K::deserialize(k, ctx),
                |ctx, k, v| Ok((k, V::deserialize(v, ctx)?)),
            )
            .collect()
    }
}
