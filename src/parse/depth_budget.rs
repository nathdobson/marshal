use std::error::Error;
use std::fmt::{Display, Formatter};
use std::marker::PhantomData;

use crate::parse::{
    AnyParser, EntryParser, EnumParser, MapParser, NewtypeParser, ParseHint, Parser,
    ParserView, ParseVariantHint, SeqParser, SomeParser,
};

pub struct DepthBudgetParser<T>(PhantomData<T>);

pub struct WithDepthBudget<T> {
    depth_budget: usize,
    inner: T,
}

#[derive(Debug)]
pub struct OverflowError;
impl Display for OverflowError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "insufficient depth budget while parsing")
    }
}

impl Error for OverflowError {}

impl<'de, T: Parser<'de>> Parser<'de> for DepthBudgetParser<T> {
    type AnyParser<'p> = WithDepthBudget<T::AnyParser<'p>> where Self: 'p;
    type SeqParser<'p> = WithDepthBudget<T::SeqParser<'p>> where Self: 'p;
    type MapParser<'p> = WithDepthBudget<T::MapParser<'p>> where Self: 'p;
    type EntryParser<'p> = WithDepthBudget<T::EntryParser<'p>> where Self: 'p;
    type EnumParser<'p> = WithDepthBudget<T::EnumParser<'p>> where Self: 'p;
    type SomeParser<'p> = WithDepthBudget<T::SomeParser<'p>> where Self: 'p;
    type NewtypeParser<'p> = WithDepthBudget<T::NewtypeParser<'p>> where Self: 'p;
}

fn annotate<'p, 'de, T: Parser<'de>>(
    depth_budget: usize,
    view: ParserView<'p, 'de, T>,
) -> anyhow::Result<ParserView<'p, 'de, DepthBudgetParser<T>>> {
    let depth_budget: Result<usize, OverflowError> =
        depth_budget.checked_sub(1).ok_or(OverflowError);
    Ok(match view {
        ParserView::Primitive(x) => ParserView::Primitive(x),
        ParserView::String(x) => ParserView::String(x),
        ParserView::Bytes(x) => ParserView::Bytes(x),
        ParserView::None => ParserView::None,
        ParserView::Some(inner) => ParserView::Some(WithDepthBudget {
            depth_budget: depth_budget?,
            inner,
        }),
        ParserView::Newtype(inner) => ParserView::Newtype(WithDepthBudget {
            depth_budget: depth_budget?,
            inner,
        }),
        ParserView::Seq(inner) => ParserView::Seq(WithDepthBudget {
            depth_budget: depth_budget?,
            inner,
        }),
        ParserView::Map(inner) => ParserView::Map(WithDepthBudget {
            depth_budget: depth_budget?,
            inner,
        }),
        ParserView::Enum(inner) => ParserView::Enum(WithDepthBudget {
            depth_budget: depth_budget?,
            inner,
        }),
    })
}

impl<T> WithDepthBudget<T> {
    pub fn new(depth_budget: usize, inner: T) -> Self {
        WithDepthBudget {
            depth_budget,
            inner,
        }
    }
}

impl<'p, 'de, T: Parser<'de>> AnyParser<'p, 'de, DepthBudgetParser<T>>
    for WithDepthBudget<T::AnyParser<'p>>
{
    fn parse(self, hint: ParseHint) -> anyhow::Result<ParserView<'p, 'de, DepthBudgetParser<T>>> {
        annotate(self.depth_budget, self.inner.parse(hint)?)
    }
}

impl<'p, 'de, T: Parser<'de>> SeqParser<'p, 'de, DepthBudgetParser<T>>
    for WithDepthBudget<T::SeqParser<'p>>
{
    fn parse_next<'p2>(
        &'p2 mut self,
    ) -> anyhow::Result<Option<WithDepthBudget<T::AnyParser<'p2>>>> {
        if let Some(inner) = self.inner.parse_next()? {
            Ok(Some(WithDepthBudget {
                depth_budget: self.depth_budget,
                inner,
            }))
        } else {
            Ok(None)
        }
    }
}

impl<'p, 'de, T: Parser<'de>> MapParser<'p, 'de, DepthBudgetParser<T>>
    for WithDepthBudget<T::MapParser<'p>>
{
    fn parse_next<'p2>(
        &'p2 mut self,
    ) -> anyhow::Result<Option<WithDepthBudget<T::EntryParser<'p2>>>> {
        if let Some(inner) = self.inner.parse_next()? {
            Ok(Some(WithDepthBudget {
                depth_budget: self.depth_budget,
                inner,
            }))
        } else {
            Ok(None)
        }
    }
}

impl<'p, 'de, T: Parser<'de>> EntryParser<'p, 'de, DepthBudgetParser<T>>
    for WithDepthBudget<T::EntryParser<'p>>
{
    fn parse_key<'p2>(&'p2 mut self) -> anyhow::Result<WithDepthBudget<T::AnyParser<'p2>>> {
        Ok(WithDepthBudget {
            depth_budget: self.depth_budget,
            inner: self.inner.parse_key()?,
        })
    }

    fn parse_value<'p2>(&'p2 mut self) -> anyhow::Result<WithDepthBudget<T::AnyParser<'p2>>> {
        Ok(WithDepthBudget {
            depth_budget: self.depth_budget,
            inner: self.inner.parse_value()?,
        })
    }

    fn parse_end(self) -> anyhow::Result<()> {
        Ok(self.inner.parse_end()?)
    }
}

impl<'p, 'de, T: Parser<'de>> EnumParser<'p, 'de, DepthBudgetParser<T>>
    for WithDepthBudget<T::EnumParser<'p>>
{
    fn parse_discriminant<'p2>(
        &'p2 mut self,
    ) -> anyhow::Result<WithDepthBudget<T::AnyParser<'p2>>> {
        Ok(WithDepthBudget {
            depth_budget: self.depth_budget,
            inner: self.inner.parse_discriminant()?,
        })
    }

    fn parse_variant<'p2>(
        &'p2 mut self,
        hint: ParseVariantHint,
    ) -> anyhow::Result<ParserView<'p2, 'de, DepthBudgetParser<T>>> {
        Ok(annotate(
            self.depth_budget,
            self.inner.parse_variant(hint)?,
        )?)
    }

    fn parse_end(self) -> anyhow::Result<()> {
        Ok(self.inner.parse_end()?)
    }
}
impl<'p, 'de, T: Parser<'de>> SomeParser<'p, 'de, DepthBudgetParser<T>>
    for WithDepthBudget<<T as Parser<'de>>::SomeParser<'p>>
{
    fn parse_some<'p2>(
        &'p2 mut self,
    ) -> anyhow::Result<<DepthBudgetParser<T> as Parser<'de>>::AnyParser<'p2>> {
        Ok(WithDepthBudget {
            depth_budget: self.depth_budget,
            inner: self.inner.parse_some()?,
        })
    }

    fn parse_end(self) -> anyhow::Result<()> {
        Ok(self.inner.parse_end()?)
    }
}
impl<'p, 'de, T: Parser<'de>> NewtypeParser<'p, 'de, DepthBudgetParser<T>>
    for WithDepthBudget<<T as Parser<'de>>::NewtypeParser<'p>>
{
    fn parse_newtype<'p2>(
        &'p2 mut self,
    ) -> anyhow::Result<<DepthBudgetParser<T> as Parser<'de>>::AnyParser<'p2>> {
        Ok(WithDepthBudget {
            depth_budget: self.depth_budget,
            inner: self.inner.parse_newtype()?,
        })
    }

    fn parse_end(self) -> anyhow::Result<()> {
        Ok(self.inner.parse_end()?)
    }
}
