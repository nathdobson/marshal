use std::collections::{BTreeMap, HashMap};
use std::hash::Hash;

use marshal_core::decode::{AnyParser, MapParser, ParseHint, Parser};

use crate::context::Context;
use crate::de::Deserialize;

impl<'de, P: Parser<'de>, K: Hash + Eq + Deserialize<'de, P>, V: Deserialize<'de, P>>
    Deserialize<'de, P> for HashMap<K, V>
{
    fn deserialize<'p>(p: P::AnyParser<'p>, ctx: &mut Context) -> anyhow::Result<Self> {
        p.parse(ParseHint::Map)?
            .try_into_map()?
            .map_into_iter(
                ctx,
                |ctx, k| K::deserialize(k, ctx),
                |ctx, k, v| Ok((k, V::deserialize(v, ctx)?)),
            )
            .collect()
    }
}


impl<'de, P: Parser<'de>, K: Ord + Deserialize<'de, P>, V: Deserialize<'de, P>>
Deserialize<'de, P> for BTreeMap<K, V>
{
    fn deserialize<'p>(p: P::AnyParser<'p>, ctx: &mut Context) -> anyhow::Result<Self> {
        p.parse(ParseHint::Map)?
            .try_into_map()?
            .map_into_iter(
                ctx,
                |ctx, k| K::deserialize(k, ctx),
                |ctx, k, v| Ok((k, V::deserialize(v, ctx)?)),
            )
            .collect()
    }
}
