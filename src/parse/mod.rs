use crate::{Primitive, PrimitiveType};
use std::fmt::{Debug, Formatter};

pub mod depth_budget;
pub mod json;
mod poison;
mod simple;

#[derive(Debug, Copy, Clone)]
pub enum ParseHint {
    Any,
    Primitive(PrimitiveType),
    String,
    Bytes,
    Option,
    UnitStruct {
        name: &'static str,
    },
    NewtypeStruct {
        name: &'static str,
    },
    Seq,
    Tuple {
        len: usize,
    },
    TupleStruct {
        name: &'static str,
        len: usize,
    },
    Map,
    Struct {
        name: &'static str,
        fields: &'static [&'static str],
    },
    Enum {
        name: &'static str,
        variants: &'static [&'static str],
    },
    Identifier,
}

pub enum ParseVariantHint {
    UnitVariant,
    NewtypeVariant,
    TupleVariant,
    StructVariant,
}

pub enum ParserView<'p, 'de, P: ?Sized + Parser<'de>>
where
    P: 'p,
{
    Primitive(Primitive),
    String(String),
    Bytes(Vec<u8>),
    None,
    Some(P::SomeParser<'p>),
    Newtype(P::NewtypeParser<'p>),
    Seq(P::SeqParser<'p>),
    Map(P::MapParser<'p>),
    Enum(P::EnumParser<'p>),
}

pub trait AnyParser<'p, 'de, P: ?Sized + Parser<'de>> {
    fn parse(self, hint: ParseHint) -> anyhow::Result<ParserView<'p, 'de, P>>;
}

pub trait SeqParser<'p, 'de, P: ?Sized + Parser<'de>> {
    fn parse_next<'p2>(&'p2 mut self) -> anyhow::Result<Option<P::AnyParser<'p2>>>;
}

pub trait MapParser<'p, 'de, P: ?Sized + Parser<'de>> {
    fn parse_next<'p2>(&'p2 mut self) -> anyhow::Result<Option<P::EntryParser<'p2>>>;
}

pub trait EntryParser<'p, 'de, P: ?Sized + Parser<'de>> {
    fn parse_key<'p2>(&'p2 mut self) -> anyhow::Result<P::AnyParser<'p2>>;
    fn parse_value<'p2>(&'p2 mut self) -> anyhow::Result<P::AnyParser<'p2>>;
    fn parse_end(self) -> anyhow::Result<()>;
}

pub trait EnumParser<'p, 'de, P: ?Sized + Parser<'de>> {
    fn parse_discriminant<'p2>(&'p2 mut self) -> anyhow::Result<P::AnyParser<'p2>>;
    fn parse_variant<'p2>(
        &'p2 mut self,
        hint: ParseVariantHint,
    ) -> anyhow::Result<ParserView<'p2, 'de, P>>;
    fn parse_end(self) -> anyhow::Result<()>;
}

pub trait SomeParser<'p, 'de, P: ?Sized + Parser<'de>> {
    fn parse_some<'p2>(&'p2 mut self) -> anyhow::Result<<P as Parser<'de>>::AnyParser<'p2>>;
    fn parse_end(self) -> anyhow::Result<()>;
}

pub trait NewtypeParser<'p, 'de, P: ?Sized + Parser<'de>> {
    fn parse_newtype<'p2>(&'p2 mut self) -> anyhow::Result<<P as Parser<'de>>::AnyParser<'p2>>;
    fn parse_end(self) -> anyhow::Result<()>;
}

pub trait Parser<'de> {
    type AnyParser<'p>: AnyParser<'p, 'de, Self>
    where
        Self: 'p;
    type SeqParser<'p>: SeqParser<'p, 'de, Self>
    where
        Self: 'p;
    type MapParser<'p>: MapParser<'p, 'de, Self>
    where
        Self: 'p;
    type EntryParser<'p>: EntryParser<'p, 'de, Self>
    where
        Self: 'p;
    type EnumParser<'p>: EnumParser<'p, 'de, Self>
    where
        Self: 'p;
    type SomeParser<'p>: SomeParser<'p, 'de, Self>
    where
        Self: 'p;
    type NewtypeParser<'p>: NewtypeParser<'p, 'de, Self>
    where
        Self: 'p;
}

impl<'p, 'de, P: Parser<'de>> Debug for ParserView<'p, 'de, P> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            ParserView::Primitive(x) => f.debug_tuple("Primitive").field(x).finish(),
            ParserView::String(x) => f.debug_tuple("String").field(x).finish(),
            ParserView::Bytes(x) => f.debug_tuple("Bytes").field(x).finish(),
            ParserView::None => f.debug_tuple("None").finish(),
            ParserView::Some(_) => f.debug_struct("Some").finish_non_exhaustive(),
            ParserView::Newtype(_) => f.debug_struct("Newtype").finish_non_exhaustive(),
            ParserView::Seq(_) => f.debug_struct("Seq").finish_non_exhaustive(),
            ParserView::Map(_) => f.debug_struct("Map").finish_non_exhaustive(),
            ParserView::Enum(_) => f.debug_struct("Enum").finish_non_exhaustive(),
        }
    }
}
