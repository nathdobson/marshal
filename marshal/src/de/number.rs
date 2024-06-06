use crate::context::Context;
use crate::de::Deserialize;
use marshal_core::parse::{AnyParser, ParseHint, Parser, ParserView};
use marshal_core::{Primitive, PrimitiveType};

impl<'de, P: Parser<'de>> Deserialize<'de, P> for u32 {
    fn deserialize<'p>(p: P::AnyParser<'p>, ctx: &mut Context) -> anyhow::Result<Self> {
        match p.parse(ParseHint::Primitive(PrimitiveType::U32))? {
            ParserView::Primitive(Primitive::U32(x)) => Ok(x),
            unexpected => unexpected.mismatch("u32")?,
        }
    }
}
