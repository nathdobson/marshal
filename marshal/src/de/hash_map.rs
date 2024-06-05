use crate::de::context::DeserializeContext;
use crate::de::Deserialize;
use crate::parse::{AnyParser, MapParser, ParseHint, Parser};
use std::collections::HashMap;
use std::hash::Hash;

impl<'de, P: Parser<'de>, K: Hash + Eq + Deserialize<'de, P>, V: Deserialize<'de, P>>
    Deserialize<'de, P> for HashMap<K, V>
{
    fn deserialize<'p>(p: P::AnyParser<'p>, ctx: &DeserializeContext) -> anyhow::Result<Self> {
        p.parse(ParseHint::Map)?
            .try_into_map()?
            .map_into_iter(
                |k| K::deserialize(k, ctx),
                |k, v| Ok((k, V::deserialize(v, ctx)?)),
            )
            .collect()
    }
}
