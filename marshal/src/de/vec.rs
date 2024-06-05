use crate::de::context::DeserializeContext;
use crate::de::Deserialize;
use crate::parse::{AnyParser, ParseHint, Parser, ParserView, SeqParser};

impl<'de, P: Parser<'de>, T: Deserialize<'de, P>> Deserialize<'de, P> for Vec<T> {
    fn deserialize<'p>(p: P::AnyParser<'p>, ctx: &DeserializeContext) -> anyhow::Result<Self> {
        match p.parse(ParseHint::Seq)? {
            ParserView::Seq(seq) => seq.seq_into_iter(|x| T::deserialize(x, ctx)).collect(),
            unexpected => unexpected.mismatch("seq")?,
        }
    }
}
