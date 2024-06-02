use crate::{AnyParser, EntryParser, EnumParser, MapParser, Parser, ParserView, SeqParser};
use std::marker::PhantomData;

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
    Some(P::AnyParser),
    Unit,
    NewType(P::AnyParser),
    Seq(P::SeqParser),
    Map(P::MapParser),
    Enum(P::DiscriminantParser),
}

pub trait SimpleParser<'de> {
    type Error;
    type AnyParser;
    type SeqParser;
    type MapParser;
    type KeyParser;
    type ValueParser;
    type DiscriminantParser;
    type VariantParser;

    fn parse_any(
        &mut self,
        any: Self::AnyParser,
    ) -> Result<SimpleParserView<'de, Self>, Self::Error>;

    fn parse_bool(
        &mut self,
        any: Self::AnyParser,
    ) -> Result<SimpleParserView<'de, Self>, Self::Error>;
    fn parse_i64(
        &mut self,
        any: Self::AnyParser,
    ) -> Result<SimpleParserView<'de, Self>, Self::Error>;
    fn parse_u64(
        &mut self,
        any: Self::AnyParser,
    ) -> Result<SimpleParserView<'de, Self>, Self::Error>;
    fn parse_f64(
        &mut self,
        any: Self::AnyParser,
    ) -> Result<SimpleParserView<'de, Self>, Self::Error>;
    fn parse_char(
        &mut self,
        any: Self::AnyParser,
    ) -> Result<SimpleParserView<'de, Self>, Self::Error>;
    fn parse_string(
        &mut self,
        any: Self::AnyParser,
    ) -> Result<SimpleParserView<'de, Self>, Self::Error>;
    fn parse_bytes(
        &mut self,
        any: Self::AnyParser,
    ) -> Result<SimpleParserView<'de, Self>, Self::Error>;
    fn parse_option(
        &mut self,
        any: Self::AnyParser,
    ) -> Result<SimpleParserView<'de, Self>, Self::Error>;
    fn parse_unit(
        &mut self,
        any: Self::AnyParser,
    ) -> Result<SimpleParserView<'de, Self>, Self::Error>;
    fn parse_unit_struct(
        &mut self,
        any: Self::AnyParser,
        name: &'static str,
    ) -> Result<SimpleParserView<'de, Self>, Self::Error>;
    fn parse_newtype_struct(
        &mut self,
        any: Self::AnyParser,
        name: &'static str,
    ) -> Result<SimpleParserView<'de, Self>, Self::Error>;
    fn parse_seq(
        &mut self,
        any: Self::AnyParser,
    ) -> Result<SimpleParserView<'de, Self>, Self::Error>;
    fn parse_tuple(
        &mut self,
        any: Self::AnyParser,
        len: usize,
    ) -> Result<SimpleParserView<'de, Self>, Self::Error>;
    fn parse_tuple_struct(
        &mut self,
        any: Self::AnyParser,
        name: &'static str,
        len: usize,
    ) -> Result<SimpleParserView<'de, Self>, Self::Error>;
    fn parse_map(
        &mut self,
        any: Self::AnyParser,
    ) -> Result<SimpleParserView<'de, Self>, Self::Error>;
    fn parse_enum(
        &mut self,
        any: Self::AnyParser,
        name: &'static str,
        variants: &'static [&'static str],
    ) -> Result<SimpleParserView<'de, Self>, Self::Error>;
    fn parse_identifier(
        &mut self,
        any: Self::AnyParser,
    ) -> Result<SimpleParserView<'de, Self>, Self::Error>;
    fn is_human_readable(&self) -> bool;

    fn parse_seq_next(
        &mut self,
        seq: &mut Self::SeqParser,
    ) -> Result<Option<Self::AnyParser>, Self::Error>;

    fn parse_map_next(
        &mut self,
        seq: &mut Self::MapParser,
    ) -> Result<Option<Self::KeyParser>, Self::Error>;

    fn parse_entry_key(
        &mut self,
        seq: Self::KeyParser,
    ) -> Result<(Self::AnyParser, Self::ValueParser), Self::Error>;

    fn parse_entry_value(&mut self, seq: Self::ValueParser)
        -> Result<Self::AnyParser, Self::Error>;

    fn parse_enum_discriminant(
        &mut self,
        e: Self::DiscriminantParser,
    ) -> Result<(Self::AnyParser, Self::VariantParser), Self::Error>;

    fn parse_enum_unit_variant(&mut self, variant: Self::VariantParser) -> Result<(), Self::Error>;
    fn parse_enum_newtype_variant(
        &mut self,
        variant: Self::VariantParser,
    ) -> Result<Self::AnyParser, Self::Error>;
    fn parse_enum_tuple_variant(
        &mut self,
        variant: Self::VariantParser,
        len: usize,
    ) -> Result<SimpleParserView<'de, Self>, Self::Error>;
    fn parse_enum_struct_variant(
        &mut self,
        variant: Self::VariantParser,
        fields: &'static [&'static str],
    ) -> Result<SimpleParserView<'de, Self>, Self::Error>;
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

impl<'de, T> Parser<'de> for SimpleParserAdapter<T>
where
    T: SimpleParser<'de>,
{
    type Error = T::Error;
    type AnyParser<'p> = SimpleAnyParser<'p, 'de,T> where T:'p;
    type SeqParser<'p> = SimpleSeqParser<'p, 'de, T> where T:'p;
    type MapParser<'p> = SimpleMapParser<'p, 'de, T> where Self: 'p;
    type EntryParser<'p> = SimpleEntryParser<'p, 'de, T> where Self: 'p;
    type EnumParser<'p> = SimpleEnumParser<'p,'de, T> where Self: 'p;
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
            SimpleParserView::Some(any) => ParserView::Some(SimpleAnyParser { this, any }),
            SimpleParserView::Unit => ParserView::Unit,
            SimpleParserView::NewType(any) => ParserView::Newtype(SimpleAnyParser { this, any }),
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
    fn parse_any(self) -> Result<ParserView<'p, 'de, SimpleParserAdapter<T>>, T::Error> {
        Ok(self.this.parse_any(self.any)?.wrap(self.this))
    }

    fn parse_bool(self) -> Result<ParserView<'p, 'de, SimpleParserAdapter<T>>, T::Error> {
        Ok(self.this.parse_bool(self.any)?.wrap(self.this))
    }

    fn parse_i64(self) -> Result<ParserView<'p, 'de, SimpleParserAdapter<T>>, T::Error> {
        Ok(self.this.parse_i64(self.any)?.wrap(self.this))
    }

    fn parse_u64(self) -> Result<ParserView<'p, 'de, SimpleParserAdapter<T>>, T::Error> {
        Ok(self.this.parse_u64(self.any)?.wrap(self.this))
    }

    fn parse_f64(self) -> Result<ParserView<'p, 'de, SimpleParserAdapter<T>>, T::Error> {
        Ok(self.this.parse_f64(self.any)?.wrap(self.this))
    }

    fn parse_char(self) -> Result<ParserView<'p, 'de, SimpleParserAdapter<T>>, T::Error> {
        Ok(self.this.parse_char(self.any)?.wrap(self.this))
    }

    fn parse_string(self) -> Result<ParserView<'p, 'de, SimpleParserAdapter<T>>, T::Error> {
        Ok(self.this.parse_string(self.any)?.wrap(self.this))
    }

    fn parse_bytes(self) -> Result<ParserView<'p, 'de, SimpleParserAdapter<T>>, T::Error> {
        Ok(self.this.parse_bytes(self.any)?.wrap(self.this))
    }

    fn parse_option(self) -> Result<ParserView<'p, 'de, SimpleParserAdapter<T>>, T::Error> {
        Ok(self.this.parse_option(self.any)?.wrap(self.this))
    }

    fn parse_unit(self) -> Result<ParserView<'p, 'de, SimpleParserAdapter<T>>, T::Error> {
        Ok(self.this.parse_unit(self.any)?.wrap(self.this))
    }

    fn parse_unit_struct(
        self,
        name: &'static str,
    ) -> Result<ParserView<'p, 'de, SimpleParserAdapter<T>>, T::Error> {
        Ok(self.this.parse_unit_struct(self.any, name)?.wrap(self.this))
    }

    fn parse_newtype_struct(
        self,
        name: &'static str,
    ) -> Result<ParserView<'p, 'de, SimpleParserAdapter<T>>, T::Error> {
        Ok(self
            .this
            .parse_newtype_struct(self.any, name)?
            .wrap(self.this))
    }

    fn parse_seq(self) -> Result<ParserView<'p, 'de, SimpleParserAdapter<T>>, T::Error> {
        Ok(self.this.parse_seq(self.any)?.wrap(self.this))
    }

    fn parse_tuple(
        self,
        len: usize,
    ) -> Result<ParserView<'p, 'de, SimpleParserAdapter<T>>, T::Error> {
        Ok(self.this.parse_tuple(self.any, len)?.wrap(self.this))
    }

    fn parse_tuple_struct(
        self,
        name: &'static str,
        len: usize,
    ) -> Result<ParserView<'p, 'de, SimpleParserAdapter<T>>, T::Error> {
        Ok(self
            .this
            .parse_tuple_struct(self.any, name, len)?
            .wrap(self.this))
    }

    fn parse_map(self) -> Result<ParserView<'p, 'de, SimpleParserAdapter<T>>, T::Error> {
        Ok(self.this.parse_map(self.any)?.wrap(self.this))
    }

    fn parse_enum(
        self,
        name: &'static str,
        variants: &'static [&'static str],
    ) -> Result<ParserView<'p, 'de, SimpleParserAdapter<T>>, T::Error> {
        Ok(self
            .this
            .parse_enum(self.any, name, variants)?
            .wrap(self.this))
    }

    fn parse_identifier(self) -> Result<ParserView<'p, 'de, SimpleParserAdapter<T>>, T::Error> {
        Ok(self.this.parse_identifier(self.any)?.wrap(self.this))
    }

    fn is_human_readable(&self) -> bool {
        self.this.is_human_readable()
    }
}

impl<'p, 'de, T> SeqParser<'p, 'de, SimpleParserAdapter<T>> for SimpleSeqParser<'p, 'de, T>
where
    T: SimpleParser<'de>,
{
    fn parse_next<'p2>(&'p2 mut self) -> Result<Option<SimpleAnyParser<'p2, 'de, T>>, T::Error> {
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
    fn parse_next<'p2>(&'p2 mut self) -> Result<Option<SimpleEntryParser<'p2, 'de, T>>, T::Error> {
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
    fn parse_key<'p2>(&'p2 mut self) -> Result<SimpleAnyParser<'p2, 'de, T>, T::Error> {
        let (key, value) = self.this.parse_entry_key(self.key.take().unwrap())?;
        self.value = Some(value);
        Ok(SimpleAnyParser {
            this: self.this,
            any: key,
        })
    }

    fn parse_value(mut self) -> Result<SimpleAnyParser<'p, 'de, T>, T::Error> {
        let value = self.value.take().unwrap();
        let value = self.this.parse_entry_value(value)?;
        Ok(SimpleAnyParser {
            this: self.this,
            any: value,
        })
    }
}

impl<'p, 'de, T> EnumParser<'p, 'de, SimpleParserAdapter<T>> for SimpleEnumParser<'p, 'de, T>
where
    T: SimpleParser<'de>,
{
    fn parse_discriminant<'p2>(&'p2 mut self) -> Result<SimpleAnyParser<'p2, 'de, T>, T::Error> {
        let (discriminant, variant) = self
            .this
            .parse_enum_discriminant(self.discriminant.take().unwrap())?;
        self.variant = Some(variant);
        Ok(SimpleAnyParser {
            this: self.this,
            any: discriminant,
        })
    }

    fn parse_unit_variant(&mut self) -> Result<(), T::Error> {
        let data = self.variant.take().unwrap();
        self.this.parse_enum_unit_variant(data)?;
        Ok(())
    }

    fn parse_newtype_variant<'p2>(&'p2 mut self) -> Result<SimpleAnyParser<'p2, 'de, T>, T::Error> {
        let data = self.variant.take().unwrap();
        let data = self.this.parse_enum_newtype_variant(data)?;
        Ok(SimpleAnyParser {
            this: self.this,
            any: data,
        })
    }

    fn parse_tuple_variant<'p2>(
        &'p2 mut self,
        len: usize,
    ) -> Result<ParserView<'p2, 'de, SimpleParserAdapter<T>>, T::Error> {
        let data = self.variant.take().unwrap();
        let data = self.this.parse_enum_tuple_variant(data, len)?;
        Ok(data.wrap(self.this))
    }

    fn parse_struct_variant<'p2>(
        &'p2 mut self,
        fields: &'static [&'static str],
    ) -> Result<ParserView<'p2, 'de, SimpleParserAdapter<T>>, T::Error> {
        let data = self.variant.take().unwrap();
        let data = self.this.parse_enum_struct_variant(data, fields)?;
        Ok(data.wrap(self.this))
    }
}
