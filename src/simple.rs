use crate::{AnyParser, AnyParserView, Parser, SeqParser};
use std::marker::PhantomData;

pub struct SimpleParserAdapter<T> {
    inner: PhantomData<T>,
}

pub enum SimpleParserView<'de, P: ?Sized + SimpleParser<'de>> {
    SeqParser(P::SeqParser),
    I64(i64),
    U64(u64),
    F64(f64),
}

pub trait SimpleParser<'de> {
    type Error;
    type AnyParser;
    type SeqParser;
    fn parse_any(
        &mut self,
        any: Self::AnyParser,
    ) -> Result<SimpleParserView<'de, Self>, Self::Error>;
    fn parse_seq_next(
        &mut self,
        seq: &mut Self::SeqParser,
    ) -> Result<SimpleParserView<'de, Self>, Self::Error>;
    fn parse_seq_end(&mut self, seq: Self::SeqParser) -> Result<(), Self::Error>;
}

pub struct SimpleParserPair<'p, T, D> {
    pub this: &'p mut T,
    pub data: D,
}

impl<'de, T> Parser<'de> for SimpleParserAdapter<T>
where
    T: SimpleParser<'de>,
{
    type Error = T::Error;
    type AnyParser<'p> = SimpleParserPair<'p, T, T::AnyParser> where T:'p;
    type SeqParser<'p> = SimpleParserPair<'p, T, T::SeqParser> where T:'p;
}

impl<'p, 'de, T> AnyParser<'p, 'de, SimpleParserAdapter<T>>
    for SimpleParserPair<'p, T, T::AnyParser>
where
    T: SimpleParser<'de>,
{
    fn parse_any(self) -> Result<AnyParserView<'p, 'de, SimpleParserAdapter<T>>, T::Error> {
        todo!()
    }
}

impl<'p, 'de, T> SeqParser<'p, 'de, SimpleParserAdapter<T>>
    for SimpleParserPair<'p, T, T::SeqParser>
where
    T: SimpleParser<'de>,
{
    fn parse_next<'p2>(&'p2 mut self) -> Result<SimpleParserPair<'p2, T, T::AnyParser>, T::Error> {
        todo!()
    }

    fn end(self) -> Result<(), T::Error> {
        todo!()
    }
}
