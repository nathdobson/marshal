use std::marker::PhantomData;

use crate::parse::{AnyParser, EntryParser, EnumParser, MapParser, NewtypeParser, ParseHint, Parser, ParserView, ParseVariantHint, SeqParser, SomeParser};

pub struct SimpleParserAdapter<T> {
    inner: PhantomData<T>,
}

pub enum SimpleParserView<'de, P: ?Sized + SimpleParser<'de>> {
    Bool(bool),
    I64(i64),
    U64(u64),
    F64(f64),
    Char(char),
    String(String),
    Bytes(Vec<u8>),
    None,
    Some(P::SomeParser),
    Unit,
    NewType(P::NewtypeParser),
    Seq(P::SeqParser),
    Map(P::MapParser),
    Enum(P::DiscriminantParser),
}

pub trait SimpleParser<'de> {
    type AnyParser;
    type SeqParser;
    type MapParser;
    type KeyParser;
    type ValueParser;
    type DiscriminantParser;
    type VariantParser;
    type SomeParser;
    type NewtypeParser;

    fn parse(
        &mut self,
        any: Self::AnyParser,
        hint: ParseHint,
    ) -> anyhow::Result<SimpleParserView<'de, Self>>;
    fn is_human_readable(&self) -> bool;

    fn parse_seq_next(
        &mut self,
        seq: &mut Self::SeqParser,
    ) -> anyhow::Result<Option<Self::AnyParser>>;

    fn parse_map_next(
        &mut self,
        map: &mut Self::MapParser,
    ) -> anyhow::Result<Option<Self::KeyParser>>;

    fn parse_entry_key(
        &mut self,
        key: Self::KeyParser,
    ) -> anyhow::Result<(Self::AnyParser, Self::ValueParser)>;

    fn parse_entry_value(
        &mut self,
        value: Self::ValueParser,
    ) -> anyhow::Result<Self::AnyParser>;

    fn parse_enum_discriminant(
        &mut self,
        e: Self::DiscriminantParser,
    ) -> anyhow::Result<(Self::AnyParser, Self::VariantParser)>;

    fn parse_enum_variant(
        &mut self,
        e: Self::VariantParser,
        hint: ParseVariantHint,
    ) -> anyhow::Result<SimpleParserView<'de, Self>>;
}

pub struct SimpleAnyParser<'p, 'de, T: SimpleParser<'de>> {
    this: &'p mut T,
    any: T::AnyParser,
}

pub struct SimpleSeqParser<'p, 'de, T: SimpleParser<'de>> {
    this: &'p mut T,
    seq: T::SeqParser,
}

pub struct SimpleMapParser<'p, 'de, T: SimpleParser<'de>> {
    this: &'p mut T,
    map: T::MapParser,
}

pub struct SimpleEntryParser<'p, 'de, T: SimpleParser<'de>> {
    this: &'p mut T,
    key: Option<T::KeyParser>,
    value: Option<T::ValueParser>,
}

pub struct SimpleEnumParser<'p, 'de, T: SimpleParser<'de>> {
    this: &'p mut T,
    discriminant: Option<T::DiscriminantParser>,
    variant: Option<T::VariantParser>,
}

pub struct SimpleSomeParser<'p, 'de, T: SimpleParser<'de>> {
    this: &'p mut T,
    some: Option<T::SomeParser>,
}

pub struct SimpleNewtypeParser<'p, 'de, T: SimpleParser<'de>> {
    this: &'p mut T,
    newtype: Option<T::NewtypeParser>,
}

impl<'de, T> Parser<'de> for SimpleParserAdapter<T>
where
    T: SimpleParser<'de>,
{
    type AnyParser<'p> = SimpleAnyParser<'p, 'de,T> where T:'p;
    type SeqParser<'p> = SimpleSeqParser<'p, 'de, T> where T:'p;
    type MapParser<'p> = SimpleMapParser<'p, 'de, T> where Self: 'p;
    type EntryParser<'p> = SimpleEntryParser<'p, 'de, T> where Self: 'p;
    type EnumParser<'p> = SimpleEnumParser<'p,'de, T> where Self: 'p;
    type SomeParser<'p> = SimpleSomeParser<'p,'de,T> where Self:'p;
    type NewtypeParser<'p> = SimpleNewtypeParser<'p,'de,T> where Self:'p;
}

impl<'de, T: SimpleParser<'de>> SimpleParserView<'de, T> {
    fn wrap<'p>(self, this: &'p mut T) -> ParserView<'p, 'de, SimpleParserAdapter<T>> {
        match self {
            SimpleParserView::Bool(x) => ParserView::Bool(x),
            SimpleParserView::I64(x) => ParserView::I64(x),
            SimpleParserView::U64(x) => ParserView::U64(x),
            SimpleParserView::F64(x) => ParserView::F64(x),
            SimpleParserView::Char(x) => ParserView::Char(x),
            SimpleParserView::String(x) => ParserView::String(x),
            SimpleParserView::Bytes(x) => ParserView::Bytes(x),
            SimpleParserView::None => ParserView::None,
            SimpleParserView::Some(some) => ParserView::Some(SimpleSomeParser {
                this,
                some: Some(some),
            }),
            SimpleParserView::Unit => ParserView::Unit,
            SimpleParserView::NewType(newtype) => ParserView::Newtype(SimpleNewtypeParser {
                this,
                newtype: Some(newtype),
            }),
            SimpleParserView::Seq(seq) => ParserView::Seq(SimpleSeqParser { this, seq }),
            SimpleParserView::Map(map) => ParserView::Map(SimpleMapParser { this, map }),
            SimpleParserView::Enum(data) => ParserView::Enum(SimpleEnumParser {
                this,
                discriminant: Some(data),
                variant: None,
            }),
        }
    }
}

impl<'p, 'de, T> AnyParser<'p, 'de, SimpleParserAdapter<T>> for SimpleAnyParser<'p, 'de, T>
where
    T: SimpleParser<'de>,
{
    fn parse(
        self,
        hint: ParseHint,
    ) -> anyhow::Result<ParserView<'p, 'de, SimpleParserAdapter<T>>> {
        Ok(self.this.parse(self.any, hint)?.wrap(self.this))
    }
}

impl<'p, 'de, T> SeqParser<'p, 'de, SimpleParserAdapter<T>> for SimpleSeqParser<'p, 'de, T>
where
    T: SimpleParser<'de>,
{
    fn parse_next<'p2>(&'p2 mut self) -> anyhow::Result<Option<SimpleAnyParser<'p2, 'de, T>>> {
        if let Some(any) = self.this.parse_seq_next(&mut self.seq)? {
            Ok(Some(SimpleAnyParser {
                this: self.this,
                any,
            }))
        } else {
            Ok(None)
        }
    }
}

impl<'p, 'de, T> MapParser<'p, 'de, SimpleParserAdapter<T>> for SimpleMapParser<'p, 'de, T>
where
    T: SimpleParser<'de>,
{
    fn parse_next<'p2>(
        &'p2 mut self,
    ) -> anyhow::Result<Option<SimpleEntryParser<'p2, 'de, T>>> {
        if let Some(data) = self.this.parse_map_next(&mut self.map)? {
            Ok(Some(SimpleEntryParser {
                this: self.this,
                key: Some(data),
                value: None,
            }))
        } else {
            Ok(None)
        }
    }
}
impl<'p, 'de, T> EntryParser<'p, 'de, SimpleParserAdapter<T>> for SimpleEntryParser<'p, 'de, T>
where
    T: SimpleParser<'de>,
{
    fn parse_key<'p2>(&'p2 mut self) -> anyhow::Result<SimpleAnyParser<'p2, 'de, T>> {
        let (key, value) = self.this.parse_entry_key(self.key.take().unwrap())?;
        self.value = Some(value);
        Ok(SimpleAnyParser {
            this: self.this,
            any: key,
        })
    }

    fn parse_value<'p2>(&'p2 mut self) -> anyhow::Result<SimpleAnyParser<'p2, 'de, T>> {
        let value = self.value.take().unwrap();
        let value = self.this.parse_entry_value(value)?;
        Ok(SimpleAnyParser {
            this: self.this,
            any: value,
        })
    }

    fn parse_end(mut self) -> anyhow::Result<()> {
        todo!()
    }
}

impl<'p, 'de, T> EnumParser<'p, 'de, SimpleParserAdapter<T>> for SimpleEnumParser<'p, 'de, T>
where
    T: SimpleParser<'de>,
{
    fn parse_discriminant<'p2>(&'p2 mut self) -> anyhow::Result<SimpleAnyParser<'p2, 'de, T>> {
        let (discriminant, variant) = self
            .this
            .parse_enum_discriminant(self.discriminant.take().unwrap())?;
        self.variant = Some(variant);
        Ok(SimpleAnyParser {
            this: self.this,
            any: discriminant,
        })
    }

    fn parse_variant<'p2>(
        &'p2 mut self,
        hint: ParseVariantHint,
    ) -> anyhow::Result<ParserView<'p2, 'de, SimpleParserAdapter<T>>> {
        let data = self
            .this
            .parse_enum_variant(self.variant.take().unwrap(), hint)?;
        Ok(data.wrap(self.this))
    }

    fn parse_end(mut self) -> anyhow::Result<()> {
        todo!()
    }
}

impl<'p, 'de, T> SomeParser<'p, 'de, SimpleParserAdapter<T>> for SimpleSomeParser<'p, 'de, T>
where
    T: SimpleParser<'de>,
{
    fn parse_some<'p2>(&'p2 mut self) -> anyhow::Result<SimpleAnyParser<'p2, 'de, T>> {
        todo!()
    }

    fn parse_end(mut self) -> anyhow::Result<()> {
        todo!()
    }
}

impl<'p, 'de, T> NewtypeParser<'p, 'de, SimpleParserAdapter<T>> for SimpleNewtypeParser<'p, 'de, T>
where
    T: SimpleParser<'de>,
{
    fn parse_newtype<'p2>(&'p2 mut self) -> anyhow::Result<SimpleAnyParser<'p2, 'de, T>> {
        todo!()
    }

    fn parse_end(mut self) -> anyhow::Result<()> {
        todo!()
    }
}

impl<'p, 'de, T: SimpleParser<'de>> SimpleAnyParser<'p, 'de, T> {
    pub fn new(parser: &'p mut T, any: T::AnyParser) -> Self {
        SimpleAnyParser { this: parser, any }
    }
}
