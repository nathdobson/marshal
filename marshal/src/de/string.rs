use crate::context::Context;
use crate::de::Deserialize;
use crate::parse::{AnyParser, ParseHint, Parser};

impl<'de, P: Parser<'de>> Deserialize<'de, P> for String {
    fn deserialize(p: P::AnyParser<'_>, _ctx: &Context) -> anyhow::Result<Self> {
        Ok(p.parse(ParseHint::String)?.try_into_string()?.into_owned())
    }
}
