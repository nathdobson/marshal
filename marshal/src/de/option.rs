use crate::context::Context;
use crate::de::Deserialize;
use marshal_core::parse::{AnyParser, ParseHint, Parser, ParserView, SomeParser};

impl<'de, P: Parser<'de>, T: Deserialize<'de, P>> Deserialize<'de, P> for Option<T> {
    fn deserialize<'p>(p: P::AnyParser<'p>, ctx: &mut Context) -> anyhow::Result<Self> {
        match p.parse(ParseHint::Option)? {
            ParserView::None => Ok(None),
            ParserView::Some(mut p) => {
                let result = Some(T::deserialize(p.parse_some()?, ctx)?);
                p.parse_end()?;
                Ok(result)
            }
            x => x.mismatch("option")?,
        }
    }
}
