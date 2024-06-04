use std::error::Error;
use std::fmt::{Display, Formatter};

use crate::de::{Deserialize, TypeMismatch};
use crate::de::context::DeserializeContext;
use crate::parse::{AnyParser, ParseHint, Parser, ParserView, SeqParser};

impl<'de, P: Parser<'de>> Deserialize<'de, P> for () {
    fn deserialize(p: P::AnyParser<'_>, _: &DeserializeContext) -> anyhow::Result<Self> {
        match p.parse(ParseHint::Unit)? {
            ParserView::Unit => Ok(()),
            _ => Err(TypeMismatch.into()),
        }
    }
}

#[derive(Debug)]
pub struct TupleTooShort;
impl Display for TupleTooShort {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "tuple too short")
    }
}
impl Error for TupleTooShort {}

#[derive(Debug)]
pub struct TupleTooLong;
impl Display for TupleTooLong {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "tuple too long")
    }
}
impl Error for TupleTooLong {}

impl<'de, P: Parser<'de>, T1: Deserialize<'de, P>> Deserialize<'de, P> for (T1,) {
    fn deserialize(
        p: P::AnyParser<'_>,
        context: &DeserializeContext,
    ) -> anyhow::Result<Self> {
        match p.parse(ParseHint::Tuple { len: 1 })? {
            ParserView::Seq(mut p) => {
                let f0 = T1::deserialize(p.parse_next()?.ok_or(TupleTooShort)?, context)?;
                if let Some(_) = p.parse_next()? {
                    return Err(TupleTooLong.into());
                }
                Ok((f0,))
            }
            _ => Err(TypeMismatch.into()),
        }
    }
}

impl<'de, P: Parser<'de>, T1: Deserialize<'de, P>, T2: Deserialize<'de, P>> Deserialize<'de, P>
    for (T1, T2)
{
    fn deserialize(
        p: P::AnyParser<'_>,
        context: &DeserializeContext,
    ) -> anyhow::Result<Self> {
        match p.parse(ParseHint::Tuple { len: 1 })? {
            ParserView::Seq(mut p) => {
                let f0 = T1::deserialize(p.parse_next()?.ok_or(TupleTooShort)?, context)?;
                let f1 = T2::deserialize(p.parse_next()?.ok_or(TupleTooShort)?, context)?;
                if let Some(_) = p.parse_next()? {
                    return Err(TupleTooLong.into());
                }
                Ok((f0, f1))
            }
            _ => Err(TypeMismatch.into()),
        }
    }
}
