use std::borrow::Cow;

use base64::prelude::BASE64_STANDARD_NO_PAD;
use base64::Engine;
use itertools::Itertools;

use marshal_core::decode::simple::{SimpleParser, SimpleParserView};
use marshal_core::decode::{DecodeHint, DecodeVariantHint};
use marshal_core::{Primitive, PrimitiveType};

use crate::parse::any::PeekType;
use crate::parse::error::JsonError;

mod any;
mod error;
pub mod full;
mod number;
mod read;
mod string;
#[cfg(test)]
mod test;

pub struct SimpleJsonParser<'de> {
    cursor: &'de [u8],
}

#[derive(Default)]
pub struct JsonAnyParser {
    must_be_string: bool,
    cannot_be_null: bool,
}

pub enum JsonSomeParser {
    Transparent { must_be_string: bool },
    Struct,
}

pub enum JsonSomeCloser {
    Transparent,
    Struct,
}

#[derive(Default)]
pub struct JsonSeqParser {
    started: bool,
}

#[derive(Default)]
pub struct JsonMapParser {
    started: bool,
}

impl<'de> SimpleParser<'de> for SimpleJsonParser<'de> {
    type AnyParser = JsonAnyParser;
    type SeqParser = JsonSeqParser;
    type MapParser = JsonMapParser;
    type KeyParser = ();
    type ValueParser = ();
    type DiscriminantParser = ();
    type VariantParser = ();
    type EnumCloser = ();
    type SomeParser = JsonSomeParser;
    type SomeCloser = JsonSomeCloser;

    fn parse(
        &mut self,
        context: Self::AnyParser,
        hint: DecodeHint,
    ) -> anyhow::Result<SimpleParserView<'de, Self>> {
        let found = self.peek_type()?;
        if context.must_be_string {
            if found != PeekType::String {
                return Err(JsonError::ExpectedString.into());
            }
        }
        match (hint, found) {
            (_, PeekType::Null) if context.cannot_be_null => {
                Err(JsonError::UnexpectedNull.into())
            }
            (DecodeHint::Option, PeekType::Map) if context.cannot_be_null => {
                self.read_exact(b'{')?;
                let key=self.read_string()?;
                match &*key{
                    "None"=>{
                        self.read_exact(b':')?;
                        self.read_null()?;
                        self.read_exact(b'}')?;
                        Ok(SimpleParserView::None)
                    },
                    "Some"=>{
                        self.read_exact(b':')?;
                        Ok(SimpleParserView::Some(JsonSomeParser::Struct))
                    },
                    _=>return Err(JsonError::BadOption.into()),
                }

            }
            (DecodeHint::Option, PeekType::Null) => {
                self.read_null()?;
                Ok(SimpleParserView::None)
            }
            (DecodeHint::Option, _) => {
                Ok(SimpleParserView::Some(JsonSomeParser::Transparent {
                    must_be_string: context.must_be_string,
                }))
            }
            (
                DecodeHint::Any
                | DecodeHint::Ignore
                | DecodeHint::UnitStruct { .. }
                | DecodeHint::Primitive(PrimitiveType::Unit)
                | DecodeHint::Tuple { len: 0 }
                | DecodeHint::Struct { name: _, fields: &[] }
                | DecodeHint::Enum { name: _, variants: &[] }
                // ignore hint
                | DecodeHint::Identifier
                | DecodeHint::Map
                | DecodeHint::Primitive(_)
                | DecodeHint::Seq
                | DecodeHint::String
                | DecodeHint::Bytes
                | DecodeHint::Tuple { .. }
                | DecodeHint::TupleStruct { .. }
                | DecodeHint::Struct { .. }
                | DecodeHint::Enum { .. }
                , PeekType::Null
            ) => {
                self.read_null()?;
                Ok(SimpleParserView::Primitive(Primitive::Unit))
            }
            (DecodeHint::Bytes, PeekType::String) => {
                Ok(SimpleParserView::Bytes(BASE64_STANDARD_NO_PAD.decode(self.read_string()?)?.into()))
            }
            (DecodeHint::Primitive(PrimitiveType::Char), PeekType::String) => {
                Ok(SimpleParserView::Primitive(Primitive::Char(
                    self.read_string()?
                        .chars()
                        .exactly_one()
                        .ok()
                        .ok_or(JsonError::TooManyChars)?,
                )))
            }
            (
                DecodeHint::Any
                | DecodeHint::Ignore
                | DecodeHint::String
                | DecodeHint::Identifier
                // ignore hint
                | DecodeHint::Primitive(
                    PrimitiveType::Unit
                    | PrimitiveType::Bool
                    | PrimitiveType::I8
                    | PrimitiveType::I16
                    | PrimitiveType::I32
                    | PrimitiveType::I64
                    | PrimitiveType::I128
                    | PrimitiveType::U8
                    | PrimitiveType::U16
                    | PrimitiveType::U32
                    | PrimitiveType::U64
                    | PrimitiveType::U128
                    | PrimitiveType::F32
                    | PrimitiveType::F64
                )
                | DecodeHint::Map
                | DecodeHint::Seq
                | DecodeHint::UnitStruct { .. }
                | DecodeHint::Tuple { .. }
                | DecodeHint::TupleStruct { .. }
                | DecodeHint::Struct { .. }
                | DecodeHint::Enum { .. },
                PeekType::String,
            ) => {
                Ok(SimpleParserView::String(Cow::Owned(self.read_string()?)))
            }
            (_, PeekType::Seq) => {
                self.read_exact(b'[')?;
                Ok(SimpleParserView::Seq(JsonSeqParser { started: false }))
            }
            (DecodeHint::Enum { .. }, PeekType::Map) => {
                self.read_exact(b'{')?;
                Ok(SimpleParserView::Enum(()))
            }
            (
                DecodeHint::Any
                | DecodeHint::Ignore
                | DecodeHint::Map
                | DecodeHint::Struct { .. }
                // ignore hint
                | DecodeHint::Primitive(_)
                | DecodeHint::UnitStruct { .. }
                | DecodeHint::Seq
                | DecodeHint::Tuple { .. }
                | DecodeHint::TupleStruct { .. }
                | DecodeHint::Bytes
                | DecodeHint::String
                | DecodeHint::Identifier { .. },
                PeekType::Map,
            ) => {
                self.read_exact(b'{')?;
                Ok(SimpleParserView::Map(JsonMapParser { started: false }))
            }
            (_, PeekType::Bool) => Ok(
                SimpleParserView::Primitive(Primitive::Bool(self.read_bool()?)),
            ),
            (DecodeHint::Primitive(PrimitiveType::I8), PeekType::Number) => Ok(
                SimpleParserView::Primitive(Primitive::I8(self.read_number()?)),
            ),
            (DecodeHint::Primitive(PrimitiveType::I16), PeekType::Number) => Ok(
                SimpleParserView::Primitive(Primitive::I16(self.read_number()?)),
            ),
            (DecodeHint::Primitive(PrimitiveType::I32), PeekType::Number) => Ok(
                SimpleParserView::Primitive(Primitive::I32(self.read_number()?)),
            ),
            (DecodeHint::Primitive(PrimitiveType::I64), PeekType::Number) => Ok(
                SimpleParserView::Primitive(Primitive::I64(self.read_number()?)),
            ),
            (DecodeHint::Primitive(PrimitiveType::I128), PeekType::Number) => Ok(
                SimpleParserView::Primitive(Primitive::I128(self.read_number()?)),
            ),
            (DecodeHint::Primitive(PrimitiveType::U8), PeekType::Number) => Ok(
                SimpleParserView::Primitive(Primitive::U8(self.read_number()?)),
            ),
            (DecodeHint::Primitive(PrimitiveType::U16), PeekType::Number) => Ok(
                SimpleParserView::Primitive(Primitive::U16(self.read_number()?)),
            ),
            (DecodeHint::Primitive(PrimitiveType::U32), PeekType::Number) => Ok(
                SimpleParserView::Primitive(Primitive::U32(self.read_number()?)),
            ),
            (DecodeHint::Primitive(PrimitiveType::Char), PeekType::Number) => Ok(
                SimpleParserView::Primitive(Primitive::Char(char::try_from(self.read_number::<u32>()?)?)),
            ),
            (DecodeHint::Primitive(PrimitiveType::U64), PeekType::Number) => Ok(
                SimpleParserView::Primitive(Primitive::U64(self.read_number()?)),
            ),
            (
                DecodeHint::Primitive(PrimitiveType::U128) | DecodeHint::Identifier,
                PeekType::Number,
            ) => Ok(SimpleParserView::Primitive(Primitive::U128(
                self.read_number()?,
            ))),
            (DecodeHint::Primitive(PrimitiveType::F32), PeekType::Number) => {
                let n = self.read_number::<f32>()?;
                if !n.is_finite() {
                    return Err(JsonError::BadNumber.into());
                }
                Ok(SimpleParserView::Primitive(Primitive::F32(n)))
            }
            (
                DecodeHint::Primitive(PrimitiveType::F64)
                | DecodeHint::Any
                | DecodeHint::Ignore
                // Ignore hint
                | DecodeHint::Map
                | DecodeHint::String
                | DecodeHint::Bytes
                | DecodeHint::UnitStruct { .. }
                | DecodeHint::Seq { .. }
                | DecodeHint::Tuple { .. }
                | DecodeHint::TupleStruct { .. }
                | DecodeHint::Struct { .. }
                | DecodeHint::Enum { .. }
                | DecodeHint::Primitive(
                    PrimitiveType::Unit
                    | PrimitiveType::Bool
                )
                ,
                PeekType::Number
            ) => {
                let n = self.read_number::<f64>()?;
                if !n.is_finite() {
                    return Err(JsonError::BadNumber.into());
                }
                Ok(SimpleParserView::Primitive(Primitive::F64(n)))
            }
        }
    }

    fn is_human_readable(&self) -> bool {
        true
    }

    fn parse_seq_next(
        &mut self,
        seq: &mut Self::SeqParser,
    ) -> anyhow::Result<Option<Self::AnyParser>> {
        if self.try_read_exact(b']')? {
            return Ok(None);
        }
        if seq.started {
            self.read_exact(b',')?;
        }
        seq.started = true;
        Ok(Some(JsonAnyParser::default()))
    }

    fn parse_map_next(
        &mut self,
        map: &mut Self::MapParser,
    ) -> anyhow::Result<Option<Self::KeyParser>> {
        if self.try_read_exact(b'}')? {
            return Ok(None);
        }
        if map.started {
            self.read_exact(b',')?;
        }
        map.started = true;
        Ok(Some(()))
    }

    fn parse_entry_key(
        &mut self,
        _: Self::KeyParser,
    ) -> anyhow::Result<(Self::AnyParser, Self::ValueParser)> {
        Ok((
            JsonAnyParser {
                must_be_string: true,
                ..JsonAnyParser::default()
            },
            (),
        ))
    }

    fn parse_entry_value(&mut self, _: Self::ValueParser) -> anyhow::Result<Self::AnyParser> {
        self.read_exact(b':')?;
        Ok(JsonAnyParser::default())
    }

    fn parse_enum_discriminant(
        &mut self,
        _: Self::DiscriminantParser,
    ) -> anyhow::Result<(Self::AnyParser, Self::VariantParser)> {
        Ok((
            JsonAnyParser {
                must_be_string: true,
                cannot_be_null: false,
            },
            (),
        ))
    }

    fn parse_enum_variant(
        &mut self,
        _: Self::VariantParser,
        hint: DecodeVariantHint,
    ) -> anyhow::Result<(SimpleParserView<'de, Self>, Self::EnumCloser)> {
        self.read_exact(b':')?;
        let hint = match hint {
            DecodeVariantHint::UnitVariant => DecodeHint::Primitive(PrimitiveType::Unit),
            DecodeVariantHint::TupleVariant { len } => DecodeHint::TupleStruct {
                name: "<enum>",
                len,
            },
            DecodeVariantHint::StructVariant { fields } => DecodeHint::Struct {
                name: "<enum>",
                fields,
            },
            DecodeVariantHint::Ignore => DecodeHint::Ignore,
        };
        Ok((
            self.parse(
                JsonAnyParser {
                    must_be_string: false,
                    cannot_be_null: false,
                },
                hint,
            )?,
            (),
        ))
    }

    fn parse_enum_end(&mut self, _: Self::EnumCloser) -> anyhow::Result<()> {
        self.read_exact(b'}')
    }

    fn parse_some_inner(
        &mut self,
        e: Self::SomeParser,
    ) -> anyhow::Result<(Self::AnyParser, Self::SomeCloser)> {
        match e {
            JsonSomeParser::Transparent { must_be_string } => Ok((
                JsonAnyParser {
                    must_be_string,
                    cannot_be_null: true,
                },
                JsonSomeCloser::Transparent,
            )),
            JsonSomeParser::Struct => Ok((
                JsonAnyParser {
                    must_be_string: false,
                    cannot_be_null: false,
                },
                JsonSomeCloser::Struct,
            )),
        }
    }

    fn parse_some_end(&mut self, p: Self::SomeCloser) -> anyhow::Result<()> {
        match p {
            JsonSomeCloser::Transparent => Ok(()),
            JsonSomeCloser::Struct => self.read_exact(b'}'),
        }
    }
}

impl<'de> SimpleJsonParser<'de> {
    pub fn new(input: &'de [u8]) -> Self {
        SimpleJsonParser { cursor: input }
    }
}
