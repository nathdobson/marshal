use marshal_core::parse::{AnyParser, ParseHint, Parser};

use crate::context::Context;
use crate::de::Deserialize;

impl<'de, P: Parser<'de>> Deserialize<'de, P> for String {
    fn deserialize(p: P::AnyParser<'_>, _ctx: &mut Context) -> anyhow::Result<Self> {
        Ok(p.parse(ParseHint::String)?.try_into_string()?.into_owned())
    }
}
