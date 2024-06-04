use itertools::Itertools;

use crate::parse::json::any::PeekType;
use crate::parse::json::error::JsonError;
use crate::parse::{ParseHint, ParseVariantHint};
use crate::parse::simple::{SimpleParser, SimpleParserView};

mod any;
mod error;
mod number;
mod read;
mod string;
mod test;
mod value;

pub struct JsonParser<'de> {
    cursor: &'de [u8],
}

#[derive(Default)]
pub struct SingletonContext {
    must_be_string: bool,
    not_null: bool,
}

pub struct JsonSeqParser {
    started: bool,
}

pub struct JsonMapParser {
    started: bool,
}

impl<'de> SimpleParser<'de> for JsonParser<'de> {
    type AnyParser = SingletonContext;
    type SeqParser = JsonSeqParser;
    type MapParser = JsonMapParser;
    type KeyParser = ();
    type ValueParser = ();
    type DiscriminantParser = ();
    type VariantParser = ();
    type SomeParser = SingletonContext;
    type NewtypeParser = ();

    fn parse(
        &mut self,
        context: Self::AnyParser,
        hint: ParseHint,
    ) -> anyhow::Result<SimpleParserView<'de, Self>> {
        let found = self.peek_type()?;
        if context.must_be_string {
            if found != PeekType::String {
                return Err(JsonError::ExpectedString.into());
            }
        }
        match (hint, found) {
            (ParseHint::Option, PeekType::Null) => {
                self.read_null()?;
                Ok(SimpleParserView::None)
            }
            (ParseHint::Option, PeekType::Map) if context.not_null => {
                self.read_exact(b'{')?;
                Ok(SimpleParserView::Some(context))
            }
            (ParseHint::Option, _) if !context.not_null => Ok(SimpleParserView::Some(context)),
            (ParseHint::Any | ParseHint::String | ParseHint::Identifier, PeekType::String) => {
                Ok(SimpleParserView::String(self.read_string()?))
            }
            (ParseHint::Bytes, PeekType::String) => {
                Ok(SimpleParserView::Bytes(self.read_string()?.into_bytes()))
            }
            (ParseHint::Char, PeekType::String) => Ok(SimpleParserView::Char(
                self.read_string()?
                    .chars()
                    .exactly_one()
                    .ok()
                    .ok_or(JsonError::TooManyChars)?,
            )),
            (
                ParseHint::Any
                | ParseHint::Seq
                | ParseHint::Tuple { .. }
                | ParseHint::TupleStruct { .. }
                | ParseHint::Bytes,
                PeekType::Seq,
            ) => {
                self.read_exact(b'[')?;
                Ok(SimpleParserView::Seq(JsonSeqParser { started: false }))
            }
            (ParseHint::Any | ParseHint::Map | ParseHint::Struct { .. }, PeekType::Map) => {
                self.read_exact(b'{')?;
                Ok(SimpleParserView::Map(JsonMapParser { started: false }))
            }
            (ParseHint::Any | ParseHint::Unit, PeekType::Null) => {
                self.read_null()?;
                Ok(SimpleParserView::Unit)
            }
            (ParseHint::Any | ParseHint::Bool, PeekType::Bool) => {
                Ok(SimpleParserView::Bool(self.read_bool()?))
            }
            (ParseHint::I64, PeekType::Number) => Ok(SimpleParserView::I64(self.read_number()?)),
            (ParseHint::U64, PeekType::Number) => Ok(SimpleParserView::U64(self.read_number()?)),
            (ParseHint::F64 | ParseHint::Any, PeekType::Number) => {
                let n = self.read_number::<f64>()?;
                if !n.is_finite() {
                    return Err(JsonError::BadNumber.into());
                }
                Ok(SimpleParserView::F64(n))
            }
            (_, _) => Err(JsonError::SchemaMismatch { hint, found }.into()),
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
        Ok(Some(SingletonContext::default()))
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
            SingletonContext {
                must_be_string: true,
                ..SingletonContext::default()
            },
            (),
        ))
    }

    fn parse_entry_value(&mut self, _: Self::ValueParser) -> anyhow::Result<Self::AnyParser> {
        self.read_exact(b':')?;
        Ok(SingletonContext::default())
    }

    fn parse_enum_discriminant(
        &mut self,
        _: Self::DiscriminantParser,
    ) -> anyhow::Result<(Self::AnyParser, Self::VariantParser)> {
        todo!()
    }

    fn parse_enum_variant(
        &mut self,
        _: Self::VariantParser,
        _: ParseVariantHint,
    ) -> anyhow::Result<SimpleParserView<'de, Self>> {
        todo!()
    }
}

impl<'de> JsonParser<'de> {
    pub fn new(input: &'de [u8]) -> Self {
        JsonParser { cursor: input }
    }
}
