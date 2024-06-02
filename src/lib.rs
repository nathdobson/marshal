#![feature(slice_take)]
#![feature(utf16_extra)]
#![deny(unused_must_use)]
#![allow(unused_mut)]
#![allow(dead_code)]

extern crate core;

mod simple;
mod text;

pub enum AnyParserView<'p, 'de, P: ?Sized + Parser<'de>>
where
    P: 'p,
{
    SeqParser(P::SeqParser<'p>),
    I64(i64),
    U64(u64),
    F64(f64),
}
pub trait AnyParser<'p, 'de, P: ?Sized + Parser<'de>> {
    fn parse_any(self) -> Result<AnyParserView<'p, 'de, P>, P::Error>;
}

pub trait SeqParser<'p, 'de, P: ?Sized + Parser<'de>> {
    fn parse_next<'p2>(&'p2 mut self) -> Result<P::AnyParser<'p2>, P::Error>;
    fn end(self) -> Result<(), P::Error>;
}

pub trait Parser<'de> {
    type Error;
    type AnyParser<'p>: AnyParser<'p, 'de, Self>
    where
        Self: 'p;
    type SeqParser<'p>: SeqParser<'p, 'de, Self>
    where
        Self: 'p;
}
