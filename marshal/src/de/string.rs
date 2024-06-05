use crate::de::context::DeserializeContext;
use crate::de::Deserialize;
use crate::parse::{AnyParser, ParseHint, Parser};

impl<'de, P: Parser<'de>> Deserialize<'de, P> for String {
    fn deserialize(p: P::AnyParser<'_>, _ctx: &DeserializeContext) -> anyhow::Result<Self> {
        p.parse(ParseHint::String)?.try_into_string()
    }
}
