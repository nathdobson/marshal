use marshal_core::parse::{AnyParser, ParseHint, Parser, ParserView, SeqParser};

use crate::context::Context;
use crate::de::Deserialize;

impl<'de, P: Parser<'de>, T: Deserialize<'de, P>> Deserialize<'de, P> for Vec<T> {
    default fn deserialize<'p>(p: P::AnyParser<'p>, ctx: &mut Context) -> anyhow::Result<Self> {
        match p.parse(ParseHint::Seq)? {
            ParserView::Seq(seq) => seq.seq_into_iter(|x| T::deserialize(x, ctx)).collect(),
            unexpected => unexpected.mismatch("seq")?,
        }
    }
}

impl<'de, P: Parser<'de>> Deserialize<'de, P> for Vec<u8> {
    fn deserialize<'p>(p: P::AnyParser<'p>, _ctx: &mut Context) -> anyhow::Result<Self> {
        match p.parse(ParseHint::Bytes)? {
            ParserView::Bytes(x) => Ok(x.into_owned()),
            unexpected => unexpected.mismatch("bytes")?,
        }
    }
}
