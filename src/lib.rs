#![feature(slice_take)]
#![feature(utf16_extra)]
#![deny(unused_must_use)]
#![allow(unused_mut)]
#![allow(dead_code)]

extern crate core;

mod simple;
mod text;

pub enum ParserView<'p, 'de, P: ?Sized + Parser<'de>>
where
    P: 'p,
{
    Bool(bool),
    I64(i64),
    U64(u64),
    F64(f64),
    Char(char),
    String(String),
    Bytes(Vec<u8>),
    None,
    Some(P::AnyParser<'p>),
    Unit,
    Newtype(P::AnyParser<'p>),
    Seq(P::SeqParser<'p>),
    Map(P::MapParser<'p>),
    Enum(P::EnumParser<'p>),
}
pub trait AnyParser<'p, 'de, P: ?Sized + Parser<'de>> {
    fn parse_any(self) -> Result<ParserView<'p, 'de, P>, P::Error>;
    fn parse_bool(self) -> Result<ParserView<'p, 'de, P>, P::Error>;
    fn parse_i64(self) -> Result<ParserView<'p, 'de, P>, P::Error>;
    fn parse_u64(self) -> Result<ParserView<'p, 'de, P>, P::Error>;
    fn parse_f64(self) -> Result<ParserView<'p, 'de, P>, P::Error>;
    fn parse_char(self) -> Result<ParserView<'p, 'de, P>, P::Error>;
    fn parse_string(self) -> Result<ParserView<'p, 'de, P>, P::Error>;
    fn parse_bytes(self) -> Result<ParserView<'p, 'de, P>, P::Error>;
    fn parse_option(self) -> Result<ParserView<'p, 'de, P>, P::Error>;
    fn parse_unit(self) -> Result<ParserView<'p, 'de, P>, P::Error>;
    fn parse_unit_struct(self, name: &'static str) -> Result<ParserView<'p, 'de, P>, P::Error>;
    fn parse_newtype_struct(self, name: &'static str) -> Result<ParserView<'p, 'de, P>, P::Error>;
    fn parse_seq(self) -> Result<ParserView<'p, 'de, P>, P::Error>;
    fn parse_tuple(self, len: usize) -> Result<ParserView<'p, 'de, P>, P::Error>;
    fn parse_tuple_struct(
        self,
        name: &'static str,
        len: usize,
    ) -> Result<ParserView<'p, 'de, P>, P::Error>;
    fn parse_map(self) -> Result<ParserView<'p, 'de, P>, P::Error>;
    fn parse_enum(
        self,
        name: &'static str,
        variants: &'static [&'static str],
    ) -> Result<ParserView<'p, 'de, P>, P::Error>;
    fn parse_identifier(self) -> Result<ParserView<'p, 'de, P>, P::Error>;
    fn is_human_readable(&self) -> bool;
}

pub trait SeqParser<'p, 'de, P: ?Sized + Parser<'de>> {
    fn parse_next<'p2>(&'p2 mut self) -> Result<Option<P::AnyParser<'p2>>, P::Error>;
}

pub trait MapParser<'p, 'de, P: ?Sized + Parser<'de>> {
    fn parse_next<'p2>(&'p2 mut self) -> Result<Option<P::EntryParser<'p2>>, P::Error>;
}

pub trait EntryParser<'p, 'de, P: ?Sized + Parser<'de>> {
    fn parse_key<'p2>(&'p2 mut self) -> Result<P::AnyParser<'p2>, P::Error>;
    fn parse_value(self) -> Result<P::AnyParser<'p>, P::Error>;
}
pub trait EnumParser<'p, 'de, P: ?Sized + Parser<'de>> {
    fn parse_discriminant<'p2>(&'p2 mut self) -> Result<P::AnyParser<'p2>, P::Error>;
    fn parse_unit_variant<'p2>(&'p2 mut self) -> Result<(), P::Error>;
    fn parse_newtype_variant<'p2>(&'p2 mut self) -> Result<P::AnyParser<'p2>, P::Error>;
    fn parse_tuple_variant<'p2>(
        &'p2 mut self,
        len: usize,
    ) -> Result<ParserView<'p2, 'de, P>, P::Error>;
    fn parse_struct_variant<'p2>(
        &'p2 mut self,
        fields: &'static [&'static str],
    ) -> Result<ParserView<'p2, 'de, P>, P::Error>;
}

pub trait Parser<'de> {
    type Error;
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
}
