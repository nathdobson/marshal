use std::borrow::Cow;

use base64::prelude::BASE64_STANDARD_NO_PAD;
use base64::Engine;
use itertools::Itertools;

use marshal_core::decode::{DecodeHint, DecodeVariantHint, SimpleDecoderView, SpecDecoder};
use marshal_core::{Primitive, PrimitiveType};

use crate::decode::any::PeekType;
use crate::decode::error::JsonDecoderError;

mod any;
mod error;
pub mod full;
mod number;
mod read;
mod string;
#[cfg(test)]
mod test;

pub struct SimpleJsonSpecDecoder<'de> {
    cursor: &'de [u8],
}

#[derive(Default)]
pub struct JsonAnyDecoder {
    must_be_string: bool,
    cannot_be_null: bool,
}

pub enum JsonSomeDecoder {
    Transparent { must_be_string: bool },
    Struct,
}

pub enum JsonSomeCloser {
    Transparent,
    Struct,
}

#[derive(Default)]
pub struct JsonSeqDecoder {
    started: bool,
}

#[derive(Default)]
pub struct JsonMapDecoder {
    started: bool,
}

pub enum JsonDiscriminantDecoder {
    Unit {
        must_be_string: bool,
        cannot_be_null: bool,
    },
    Map,
}

pub enum JsonVariantDecoder {
    Unit,
    Map,
}

pub enum JsonEnumCloser {
    Unit,
    Map,
}

impl<'de> SpecDecoder<'de> for SimpleJsonSpecDecoder<'de> {
    type AnyDecoder = JsonAnyDecoder;
    type SeqDecoder = JsonSeqDecoder;
    type MapDecoder = JsonMapDecoder;
    type KeyDecoder = ();
    type ValueDecoder = ();
    type DiscriminantDecoder = JsonDiscriminantDecoder;
    type VariantDecoder = JsonVariantDecoder;
    type EnumCloser = JsonEnumCloser;
    type SomeDecoder = JsonSomeDecoder;
    type SomeCloser = JsonSomeCloser;

    fn decode(
        &mut self,
        context: Self::AnyDecoder,
        hint: DecodeHint,
    ) -> anyhow::Result<SimpleDecoderView<'de, Self>> {
        let found = self.peek_type()?;
        if context.must_be_string {
            if found != PeekType::String {
                return Err(JsonDecoderError::ExpectedString.into());
            }
        }
        match (hint, found) {
            (_, PeekType::Null) if context.cannot_be_null => {
                Err(JsonDecoderError::UnexpectedNull.into())
            }
            (DecodeHint::Option, PeekType::Map) if context.cannot_be_null => {
                self.read_exact(b'{')?;
                let key = self.read_string()?;
                match &*key {
                    "None" => {
                        self.read_exact(b':')?;
                        self.read_null()?;
                        self.read_exact(b'}')?;
                        Ok(SimpleDecoderView::None)
                    }
                    "Some" => {
                        self.read_exact(b':')?;
                        Ok(SimpleDecoderView::Some(JsonSomeDecoder::Struct))
                    }
                    _ => return Err(JsonDecoderError::BadOption.into()),
                }
            }
            (DecodeHint::Option, PeekType::Null) => {
                self.read_null()?;
                Ok(SimpleDecoderView::None)
            }
            (DecodeHint::Option, _) => {
                Ok(SimpleDecoderView::Some(JsonSomeDecoder::Transparent {
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
                Ok(SimpleDecoderView::Primitive(Primitive::Unit))
            }
            (DecodeHint::Bytes, PeekType::String) => {
                Ok(SimpleDecoderView::Bytes(BASE64_STANDARD_NO_PAD.decode(self.read_string()?)?.into()))
            }
            (DecodeHint::Primitive(PrimitiveType::Char), PeekType::String) => {
                Ok(SimpleDecoderView::Primitive(Primitive::Char(
                    self.read_string()?
                        .chars()
                        .exactly_one()
                        .ok()
                        .ok_or(JsonDecoderError::TooManyChars)?,
                )))
            }
            (DecodeHint::Primitive(prim@(PrimitiveType::Unit
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
                                   | PrimitiveType::F64)),
                PeekType::String) if context.must_be_string => {
                Ok(SimpleDecoderView::Primitive(self.read_prim_from_str(prim)?))
            },
            (DecodeHint::Enum { .. }, PeekType::String) => {
                Ok(SimpleDecoderView::Enum(JsonDiscriminantDecoder::Unit{
                    must_be_string:context.must_be_string,
                    cannot_be_null:context.must_be_string
                }))
            },
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
                | DecodeHint::Struct { .. },
                PeekType::String,
            ) => {
                Ok(SimpleDecoderView::String(Cow::Owned(self.read_string()?)))
            }
            (
                DecodeHint::Primitive(PrimitiveType::Unit)
                | DecodeHint::UnitStruct { .. },
                PeekType::Seq
            ) if context.cannot_be_null =>{
                self.read_exact(b'[')?;
                self.read_exact(b']')?;
                Ok(SimpleDecoderView::Primitive(Primitive::Unit))
            }
            (_, PeekType::Seq) => {
                self.read_exact(b'[')?;
                Ok(SimpleDecoderView::Seq(JsonSeqDecoder { started: false }))
            }
            (DecodeHint::Enum { .. }, PeekType::Map) => {
                self.read_exact(b'{')?;
                Ok(SimpleDecoderView::Enum(JsonDiscriminantDecoder::Map))
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
                Ok(SimpleDecoderView::Map(JsonMapDecoder { started: false }))
            }
            (_, PeekType::Bool) => Ok(
                SimpleDecoderView::Primitive(Primitive::Bool(self.read_bool()?)),
            ),
            (DecodeHint::Primitive(PrimitiveType::I8), PeekType::Number) => Ok(
                SimpleDecoderView::Primitive(Primitive::I8(self.read_number()?)),
            ),
            (DecodeHint::Primitive(PrimitiveType::I16), PeekType::Number) => Ok(
                SimpleDecoderView::Primitive(Primitive::I16(self.read_number()?)),
            ),
            (DecodeHint::Primitive(PrimitiveType::I32), PeekType::Number) => Ok(
                SimpleDecoderView::Primitive(Primitive::I32(self.read_number()?)),
            ),
            (DecodeHint::Primitive(PrimitiveType::I64), PeekType::Number) => Ok(
                SimpleDecoderView::Primitive(Primitive::I64(self.read_number()?)),
            ),
            (DecodeHint::Primitive(PrimitiveType::I128), PeekType::Number) => Ok(
                SimpleDecoderView::Primitive(Primitive::I128(self.read_number()?)),
            ),
            (DecodeHint::Primitive(PrimitiveType::U8), PeekType::Number) => Ok(
                SimpleDecoderView::Primitive(Primitive::U8(self.read_number()?)),
            ),
            (DecodeHint::Primitive(PrimitiveType::U16), PeekType::Number) => Ok(
                SimpleDecoderView::Primitive(Primitive::U16(self.read_number()?)),
            ),
            (DecodeHint::Primitive(PrimitiveType::U32), PeekType::Number) => Ok(
                SimpleDecoderView::Primitive(Primitive::U32(self.read_number()?)),
            ),
            (DecodeHint::Primitive(PrimitiveType::Char), PeekType::Number) => Ok(
                SimpleDecoderView::Primitive(Primitive::Char(char::try_from(self.read_number::<u32>()?)?)),
            ),
            (DecodeHint::Primitive(PrimitiveType::U64), PeekType::Number) => Ok(
                SimpleDecoderView::Primitive(Primitive::U64(self.read_number()?)),
            ),
            (
                DecodeHint::Primitive(PrimitiveType::U128) | DecodeHint::Identifier,
                PeekType::Number,
            ) => Ok(SimpleDecoderView::Primitive(Primitive::U128(
                self.read_number()?,
            ))),
            (DecodeHint::Primitive(PrimitiveType::F32), PeekType::Number) => {
                let n = self.read_number::<f32>()?;
                if !n.is_finite() {
                    return Err(JsonDecoderError::BadNumber.into());
                }
                Ok(SimpleDecoderView::Primitive(Primitive::F32(n)))
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
                    return Err(JsonDecoderError::BadNumber.into());
                }
                Ok(SimpleDecoderView::Primitive(Primitive::F64(n)))
            }
        }
    }

    fn is_human_readable(&self) -> bool {
        true
    }

    fn decode_seq_next(
        &mut self,
        seq: &mut Self::SeqDecoder,
    ) -> anyhow::Result<Option<Self::AnyDecoder>> {
        if self.try_read_exact(b']')? {
            return Ok(None);
        }
        if seq.started {
            self.read_exact(b',')?;
        }
        seq.started = true;
        Ok(Some(JsonAnyDecoder::default()))
    }

    fn decode_seq_end(&mut self, _seq: Self::SeqDecoder) -> anyhow::Result<()> {
        Ok(())
    }

    fn decode_map_next(
        &mut self,
        map: &mut Self::MapDecoder,
    ) -> anyhow::Result<Option<Self::KeyDecoder>> {
        if self.try_read_exact(b'}')? {
            return Ok(None);
        }
        if map.started {
            self.read_exact(b',')?;
        }
        map.started = true;
        Ok(Some(()))
    }

    fn decode_map_end(&mut self, _map: Self::MapDecoder) -> anyhow::Result<()> {
        Ok(())
    }

    fn decode_entry_key(
        &mut self,
        _: Self::KeyDecoder,
    ) -> anyhow::Result<(Self::AnyDecoder, Self::ValueDecoder)> {
        Ok((
            JsonAnyDecoder {
                must_be_string: true,
                ..JsonAnyDecoder::default()
            },
            (),
        ))
    }

    fn decode_entry_value(&mut self, _: Self::ValueDecoder) -> anyhow::Result<Self::AnyDecoder> {
        self.read_exact(b':')?;
        Ok(JsonAnyDecoder::default())
    }

    fn decode_enum_discriminant(
        &mut self,
        disc: Self::DiscriminantDecoder,
    ) -> anyhow::Result<(Self::AnyDecoder, Self::VariantDecoder)> {
        match disc {
            JsonDiscriminantDecoder::Unit {
                must_be_string,
                cannot_be_null,
            } => Ok((
                JsonAnyDecoder {
                    must_be_string,
                    cannot_be_null,
                },
                JsonVariantDecoder::Unit,
            )),
            JsonDiscriminantDecoder::Map => Ok((
                JsonAnyDecoder {
                    must_be_string: true,
                    cannot_be_null: false,
                },
                JsonVariantDecoder::Map,
            )),
        }
    }

    fn decode_enum_variant(
        &mut self,
        decoder: Self::VariantDecoder,
        hint: DecodeVariantHint,
    ) -> anyhow::Result<(SimpleDecoderView<'de, Self>, Self::EnumCloser)> {
        match decoder {
            JsonVariantDecoder::Unit => Ok((
                SimpleDecoderView::Primitive(Primitive::Unit),
                JsonEnumCloser::Unit,
            )),
            JsonVariantDecoder::Map => {
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
                    self.decode(
                        JsonAnyDecoder {
                            must_be_string: false,
                            cannot_be_null: false,
                        },
                        hint,
                    )?,
                    JsonEnumCloser::Map,
                ))
            }
        }
    }

    fn decode_enum_end(&mut self, decoder: Self::EnumCloser) -> anyhow::Result<()> {
        match decoder {
            JsonEnumCloser::Unit => Ok(()),
            JsonEnumCloser::Map => self.read_exact(b'}'),
        }
    }

    fn decode_some_inner(
        &mut self,
        e: Self::SomeDecoder,
    ) -> anyhow::Result<(Self::AnyDecoder, Self::SomeCloser)> {
        match e {
            JsonSomeDecoder::Transparent { must_be_string } => Ok((
                JsonAnyDecoder {
                    must_be_string,
                    cannot_be_null: true,
                },
                JsonSomeCloser::Transparent,
            )),
            JsonSomeDecoder::Struct => Ok((
                JsonAnyDecoder {
                    must_be_string: false,
                    cannot_be_null: false,
                },
                JsonSomeCloser::Struct,
            )),
        }
    }

    fn decode_some_end(&mut self, p: Self::SomeCloser) -> anyhow::Result<()> {
        match p {
            JsonSomeCloser::Transparent => Ok(()),
            JsonSomeCloser::Struct => self.read_exact(b'}'),
        }
    }
}

impl<'de> SimpleJsonSpecDecoder<'de> {
    pub fn new(input: &'de [u8]) -> Self {
        SimpleJsonSpecDecoder { cursor: input }
    }
}
