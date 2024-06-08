use std::borrow::Cow;
use std::marker::PhantomData;

use crate::decode::{
    AnyDecoder, EntryDecoder, EnumDecoder, MapDecoder, DecodeHint, DecodeVariantHint, Decoder, DecoderView,
    SeqDecoder, SomeDecoder,
};
use crate::Primitive;

pub struct SimpleParserAdapter<T> {
    inner: PhantomData<T>,
}

pub enum SimpleParserView<'de, P: ?Sized + SimpleParser<'de>> {
    Primitive(Primitive),
    String(Cow<'de, str>),
    Bytes(Cow<'de, [u8]>),
    None,
    Some(P::SomeParser),
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
    type EnumCloser;
    type SomeParser;
    type SomeCloser;

    fn parse(
        &mut self,
        any: Self::AnyParser,
        hint: DecodeHint,
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

    fn parse_entry_value(&mut self, value: Self::ValueParser) -> anyhow::Result<Self::AnyParser>;

    fn parse_enum_discriminant(
        &mut self,
        e: Self::DiscriminantParser,
    ) -> anyhow::Result<(Self::AnyParser, Self::VariantParser)>;

    fn parse_enum_variant(
        &mut self,
        e: Self::VariantParser,
        hint: DecodeVariantHint,
    ) -> anyhow::Result<(SimpleParserView<'de, Self>, Self::EnumCloser)>;

    fn parse_enum_end(&mut self, e: Self::EnumCloser) -> anyhow::Result<()>;

    fn parse_some_inner(
        &mut self,
        e: Self::SomeParser,
    ) -> anyhow::Result<(Self::AnyParser, Self::SomeCloser)>;

    fn parse_some_end(&mut self, p: Self::SomeCloser) -> anyhow::Result<()>;
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
    closer: Option<T::EnumCloser>,
}

pub struct SimpleSomeParser<'p, 'de, T: SimpleParser<'de>> {
    this: &'p mut T,
    some_parser: Option<T::SomeParser>,
    some_closer: Option<T::SomeCloser>,
}

impl<'de, T> Decoder<'de> for SimpleParserAdapter<T>
where
    T: SimpleParser<'de>,
{
    type AnyDecoder<'p> = SimpleAnyParser<'p, 'de,T> where T:'p;
    type SeqDecoder<'p> = SimpleSeqParser<'p, 'de, T> where T:'p;
    type MapDecoder<'p> = SimpleMapParser<'p, 'de, T> where Self: 'p;
    type EntryDecoder<'p> = SimpleEntryParser<'p, 'de, T> where Self: 'p;
    type EnumDecoder<'p> = SimpleEnumParser<'p,'de, T> where Self: 'p;
    type SomeDecoder<'p> = SimpleSomeParser<'p,'de,T> where Self:'p;
}

impl<'de, T: SimpleParser<'de>> SimpleParserView<'de, T> {
    fn wrap<'p>(self, this: &'p mut T) -> DecoderView<'p, 'de, SimpleParserAdapter<T>> {
        match self {
            SimpleParserView::Primitive(x) => DecoderView::Primitive(x),
            SimpleParserView::String(x) => DecoderView::String(x),
            SimpleParserView::Bytes(x) => DecoderView::Bytes(x),
            SimpleParserView::None => DecoderView::None,
            SimpleParserView::Some(some) => DecoderView::Some(SimpleSomeParser {
                this,
                some_parser: Some(some),
                some_closer: None,
            }),
            SimpleParserView::Seq(seq) => DecoderView::Seq(SimpleSeqParser { this, seq }),
            SimpleParserView::Map(map) => DecoderView::Map(SimpleMapParser { this, map }),
            SimpleParserView::Enum(data) => DecoderView::Enum(SimpleEnumParser {
                this,
                discriminant: Some(data),
                variant: None,
                closer: None,
            }),
        }
    }
}

impl<'p, 'de, T> AnyDecoder<'p, 'de, SimpleParserAdapter<T>> for SimpleAnyParser<'p, 'de, T>
where
    T: SimpleParser<'de>,
{
    fn decode(self, hint: DecodeHint) -> anyhow::Result<DecoderView<'p, 'de, SimpleParserAdapter<T>>> {
        Ok(self.this.parse(self.any, hint)?.wrap(self.this))
    }
}

impl<'p, 'de, T> SeqDecoder<'p, 'de, SimpleParserAdapter<T>> for SimpleSeqParser<'p, 'de, T>
where
    T: SimpleParser<'de>,
{
    fn decode_next<'p2>(&'p2 mut self) -> anyhow::Result<Option<SimpleAnyParser<'p2, 'de, T>>> {
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

impl<'p, 'de, T> MapDecoder<'p, 'de, SimpleParserAdapter<T>> for SimpleMapParser<'p, 'de, T>
where
    T: SimpleParser<'de>,
{
    fn decode_next<'p2>(&'p2 mut self) -> anyhow::Result<Option<SimpleEntryParser<'p2, 'de, T>>> {
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
impl<'p, 'de, T> EntryDecoder<'p, 'de, SimpleParserAdapter<T>> for SimpleEntryParser<'p, 'de, T>
where
    T: SimpleParser<'de>,
{
    fn decode_key<'p2>(&'p2 mut self) -> anyhow::Result<SimpleAnyParser<'p2, 'de, T>> {
        let (key, value) = self.this.parse_entry_key(self.key.take().unwrap())?;
        self.value = Some(value);
        Ok(SimpleAnyParser {
            this: self.this,
            any: key,
        })
    }

    fn decode_value<'p2>(&'p2 mut self) -> anyhow::Result<SimpleAnyParser<'p2, 'de, T>> {
        let value = self.value.take().unwrap();
        let value = self.this.parse_entry_value(value)?;
        Ok(SimpleAnyParser {
            this: self.this,
            any: value,
        })
    }

    fn decode_end(mut self) -> anyhow::Result<()> {
        Ok(())
    }
}

impl<'p, 'de, T> EnumDecoder<'p, 'de, SimpleParserAdapter<T>> for SimpleEnumParser<'p, 'de, T>
where
    T: SimpleParser<'de>,
{
    fn decode_discriminant<'p2>(&'p2 mut self) -> anyhow::Result<SimpleAnyParser<'p2, 'de, T>> {
        let (discriminant, variant) = self
            .this
            .parse_enum_discriminant(self.discriminant.take().unwrap())?;
        self.variant = Some(variant);
        Ok(SimpleAnyParser {
            this: self.this,
            any: discriminant,
        })
    }

    fn decode_variant<'p2>(
        &'p2 mut self,
        hint: DecodeVariantHint,
    ) -> anyhow::Result<DecoderView<'p2, 'de, SimpleParserAdapter<T>>> {
        let (data, closer) = self
            .this
            .parse_enum_variant(self.variant.take().unwrap(), hint)?;
        self.closer = Some(closer);
        Ok(data.wrap(self.this))
    }

    fn decode_end(mut self) -> anyhow::Result<()> {
        self.this.parse_enum_end(self.closer.take().unwrap())
    }
}

impl<'p, 'de, T> SomeDecoder<'p, 'de, SimpleParserAdapter<T>> for SimpleSomeParser<'p, 'de, T>
where
    T: SimpleParser<'de>,
{
    fn decode_some<'p2>(&'p2 mut self) -> anyhow::Result<SimpleAnyParser<'p2, 'de, T>> {
        let (any, closer) = self
            .this
            .parse_some_inner(self.some_parser.take().unwrap())?;
        self.some_closer = Some(closer);
        Ok(SimpleAnyParser::new(self.this, any))
    }

    fn decode_end(mut self) -> anyhow::Result<()> {
        self.this.parse_some_end(self.some_closer.take().unwrap())
    }
}

impl<'p, 'de, T: SimpleParser<'de>> SimpleAnyParser<'p, 'de, T> {
    pub fn new(parser: &'p mut T, any: T::AnyParser) -> Self {
        SimpleAnyParser { this: parser, any }
    }
}
