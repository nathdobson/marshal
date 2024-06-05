use crate::to_from_vu128::{Array, ToFromVu128};
use marshal::parse::simple::{SimpleParser, SimpleParserView};
use marshal::parse::{ParseHint, ParseVariantHint};
use std::fmt::{Debug, Display, Formatter};

pub struct BinReader<'de> {
    content: &'de [u8],
}

#[derive(Debug)]
pub enum BinError {
    Eof,
}

impl Display for BinError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        Debug::fmt(self, f)
    }
}

impl std::error::Error for BinError {}

impl<'de> BinReader<'de> {
    pub fn read_count(&mut self, count: usize) -> anyhow::Result<&'de [u8]> {
        Ok(self.content.take(..count).ok_or(BinError::Eof)?)
    }
    pub fn read_vu128<T: ToFromVu128>(&mut self) -> anyhow::Result<T> {
        let (value, count) = T::decode_vu128(T::Buffer::try_from_slice(
            &self.content[..T::Buffer::ARRAY_LEN],
        )?);
        self.content = self.content.take(..count).ok_or(BinError::Eof)?;
        Ok(value)
    }
}

impl<'de> SimpleParser<'de> for BinReader<'de> {
    type AnyParser = ();
    type SeqParser = ();
    type MapParser = ();
    type KeyParser = ();
    type ValueParser = ();
    type DiscriminantParser = ();
    type VariantParser = ();
    type SomeParser = ();

    fn parse(
        &mut self,
        any: Self::AnyParser,
        hint: ParseHint,
    ) -> anyhow::Result<SimpleParserView<'de, Self>> {
        todo!()
    }

    fn is_human_readable(&self) -> bool {
        todo!()
    }

    fn parse_seq_next(
        &mut self,
        seq: &mut Self::SeqParser,
    ) -> anyhow::Result<Option<Self::AnyParser>> {
        todo!()
    }

    fn parse_map_next(
        &mut self,
        map: &mut Self::MapParser,
    ) -> anyhow::Result<Option<Self::KeyParser>> {
        todo!()
    }

    fn parse_entry_key(
        &mut self,
        key: Self::KeyParser,
    ) -> anyhow::Result<(Self::AnyParser, Self::ValueParser)> {
        todo!()
    }

    fn parse_entry_value(&mut self, value: Self::ValueParser) -> anyhow::Result<Self::AnyParser> {
        todo!()
    }

    fn parse_enum_discriminant(
        &mut self,
        e: Self::DiscriminantParser,
    ) -> anyhow::Result<(Self::AnyParser, Self::VariantParser)> {
        todo!()
    }

    fn parse_enum_variant(
        &mut self,
        e: Self::VariantParser,
        hint: ParseVariantHint,
    ) -> anyhow::Result<SimpleParserView<'de, Self>> {
        todo!()
    }
}
