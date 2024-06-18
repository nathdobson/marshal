use std::collections::HashMap;
use std::fmt::{Debug, Display, Formatter};

use by_address::ByAddress;

use marshal_core::encode::Encoder;
use marshal_core::Primitive;

use crate::{TypeTag, VU128_MAX_PADDING};
use crate::to_from_vu128::{Array, ToFromVu128};

pub mod full;

pub struct BinEncoderSchema {
    enum_def_indexes: HashMap<ByAddress<&'static [&'static str]>, usize>,
}

impl BinEncoderSchema {
    pub fn new() -> Self {
        BinEncoderSchema {
            enum_def_indexes: HashMap::new(),
        }
    }
}

#[derive(Debug)]
pub enum BinEncoderError {
    MissingLen,
}

impl Display for BinEncoderError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            BinEncoderError::MissingLen => write!(f, "cannot encode repeated data without length"),
        }
    }
}

impl std::error::Error for BinEncoderError {}

pub struct SimpleBinEncoder<'s> {
    output: Vec<u8>,
    schema: &'s mut BinEncoderSchema,
}

impl<'s> SimpleBinEncoder<'s> {
    pub fn new(schema: &mut BinEncoderSchema) -> SimpleBinEncoder {
        SimpleBinEncoder {
            output: vec![],
            schema,
        }
    }
    pub fn end(mut self) -> anyhow::Result<Vec<u8>> {
        //pad to maximum vu128
        self.output.resize(self.output.len() + VU128_MAX_PADDING, 0);
        Ok(self.output)
    }
    pub fn write_raw(&mut self, value: &[u8]) -> anyhow::Result<()> {
        self.output.extend_from_slice(value);
        Ok(())
    }
    pub fn write_vu128<T: ToFromVu128>(&mut self, value: T) -> anyhow::Result<()> {
        let start = self.output.len();
        self.output.resize(start + T::Buffer::ARRAY_LEN, 0);
        let written = T::encode_vu128(
            T::Buffer::try_from_slice_mut(&mut self.output[start..]).unwrap(),
            value,
        );
        self.output.resize(start + written, 0);
        Ok(())
    }
    pub fn write_tag(&mut self, tag: TypeTag) -> anyhow::Result<()> {
        self.output.push(tag as u8);
        Ok(())
    }
    pub fn write_usize(&mut self, value: usize) -> anyhow::Result<()> {
        self.write_vu128(value as u64)
    }
    pub fn write_byte_slice(&mut self, value: &[u8]) -> anyhow::Result<()> {
        self.write_usize(value.len())?;
        self.write_raw(value)?;
        Ok(())
    }

    pub fn write_str_slice(&mut self, value: &str) -> anyhow::Result<()> {
        self.write_byte_slice(value.as_bytes())
    }
    pub fn get_or_write_enum_def(
        &mut self,
        variants: &'static [&'static str],
    ) -> anyhow::Result<usize> {
        if let Some(index) = self.schema.enum_def_indexes.get(&ByAddress(variants)) {
            return Ok(*index);
        }
        self.write_tag(TypeTag::EnumDef)?;
        self.write_usize(variants.len())?;
        for x in variants {
            self.write_str_slice(x)?;
        }
        let new_index = self.schema.enum_def_indexes.len();
        self.schema
            .enum_def_indexes
            .insert(ByAddress(variants), new_index);
        Ok(new_index)
    }
}

impl<'s> Encoder for SimpleBinEncoder<'s> {
    type AnyEncoder = ();
    type SomeCloser = ();
    type TupleEncoder = ();
    type SeqEncoder = ();
    type MapEncoder = ();
    type ValueEncoder = ();
    type EntryCloser = ();
    type TupleStructEncoder = ();
    type StructEncoder = ();
    type TupleVariantEncoder = ();
    type StructVariantEncoder = ();

    fn encode_prim(&mut self, _any: Self::AnyEncoder, prim: Primitive) -> anyhow::Result<()> {
        match prim {
            Primitive::Unit => {
                self.write_tag(TypeTag::Unit)?;
            }
            Primitive::Bool(x) => {
                self.write_tag(TypeTag::Bool)?;
                self.write_vu128(x)?;
            }
            Primitive::I8(x) => {
                self.write_tag(TypeTag::I8)?;
                self.write_vu128(x)?;
            }
            Primitive::I16(x) => {
                self.write_tag(TypeTag::I16)?;
                self.write_vu128(x)?;
            }
            Primitive::I32(x) => {
                self.write_tag(TypeTag::I32)?;
                self.write_vu128(x)?;
            }
            Primitive::I64(x) => {
                self.write_tag(TypeTag::I64)?;
                self.write_vu128(x)?;
            }
            Primitive::I128(x) => {
                self.write_tag(TypeTag::I128)?;
                self.write_vu128(x)?;
            }
            Primitive::U8(x) => {
                self.write_tag(TypeTag::U8)?;
                self.write_vu128(x)?;
            }
            Primitive::U16(x) => {
                self.write_tag(TypeTag::U16)?;
                self.write_vu128(x)?;
            }
            Primitive::U32(x) => {
                self.write_tag(TypeTag::U32)?;
                self.write_vu128(x)?;
            }
            Primitive::U64(x) => {
                self.write_tag(TypeTag::U64)?;
                self.write_vu128(x)?;
            }
            Primitive::U128(x) => {
                self.write_tag(TypeTag::U128)?;
                self.write_vu128(x)?;
            }
            Primitive::F32(x) => {
                self.write_tag(TypeTag::F32)?;
                self.write_vu128(x)?;
            }
            Primitive::F64(x) => {
                self.write_tag(TypeTag::F64)?;
                self.write_vu128(x)?;
            }
            Primitive::Char(x) => {
                self.write_tag(TypeTag::Char)?;
                self.write_vu128(x as u32)?;
            }
        }
        Ok(())
    }

    fn encode_str(&mut self, _any: Self::AnyEncoder, s: &str) -> anyhow::Result<()> {
        self.write_tag(TypeTag::String)?;
        self.write_str_slice(s)?;
        Ok(())
    }

    fn encode_bytes(&mut self, _any: Self::AnyEncoder, s: &[u8]) -> anyhow::Result<()> {
        self.write_tag(TypeTag::Bytes)?;
        self.write_byte_slice(s)?;
        Ok(())
    }

    fn encode_none(&mut self, _any: Self::AnyEncoder) -> anyhow::Result<()> {
        self.write_tag(TypeTag::None)?;
        Ok(())
    }

    fn encode_some(
        &mut self,
        _any: Self::AnyEncoder,
    ) -> anyhow::Result<(Self::AnyEncoder, Self::SomeCloser)> {
        self.write_tag(TypeTag::Some)?;
        Ok(((), ()))
    }

    fn encode_unit_struct(
        &mut self,
        _any: Self::AnyEncoder,
        _name: &'static str,
    ) -> anyhow::Result<()> {
        self.write_tag(TypeTag::UnitStruct)
    }

    fn encode_tuple_struct(
        &mut self,
        _any: Self::AnyEncoder,
        _name: &'static str,
        len: usize,
    ) -> anyhow::Result<Self::TupleStructEncoder> {
        self.write_tag(TypeTag::TupleStruct)?;
        self.write_usize(len)?;
        Ok(())
    }

    fn encode_struct(
        &mut self,
        _any: Self::AnyEncoder,
        _name: &'static str,
        fields: &'static [&'static str],
    ) -> anyhow::Result<Self::StructEncoder> {
        let def = self.get_or_write_enum_def(fields)?;
        self.write_tag(TypeTag::Struct)?;
        self.write_usize(def)?;
        Ok(())
    }

    fn encode_unit_variant(
        &mut self,
        _any: Self::AnyEncoder,
        _name: &'static str,
        variants: &'static [&'static str],
        variant_index: usize,
    ) -> anyhow::Result<()> {
        let enum_def = self.get_or_write_enum_def(variants)?;
        self.write_tag(TypeTag::Enum)?;
        self.write_usize(enum_def)?;
        self.write_usize(variant_index)?;
        self.write_tag(TypeTag::Unit)?;
        Ok(())
    }

    fn encode_tuple_variant(
        &mut self,
        _any: Self::AnyEncoder,
        _name: &'static str,
        variants: &'static [&'static str],
        variant_index: usize,
        len: usize,
    ) -> anyhow::Result<Self::TupleVariantEncoder> {
        let enum_def = self.get_or_write_enum_def(variants)?;
        self.write_tag(TypeTag::Enum)?;
        self.write_usize(enum_def)?;
        self.write_usize(variant_index)?;
        self.write_tag(TypeTag::TupleStruct)?;
        self.write_usize(len)?;
        Ok(())
    }

    fn encode_struct_variant(
        &mut self,
        _any: Self::AnyEncoder,
        _name: &'static str,
        variants: &'static [&'static str],
        variant_index: usize,
        fields: &'static [&'static str],
    ) -> anyhow::Result<Self::StructVariantEncoder> {
        let variant_def = self.get_or_write_enum_def(variants)?;
        let field_def = self.get_or_write_enum_def(fields)?;
        self.write_tag(TypeTag::Enum)?;
        self.write_usize(variant_def)?;
        self.write_usize(variant_index)?;
        self.write_tag(TypeTag::Struct)?;
        self.write_usize(field_def)?;
        Ok(())
    }

    fn encode_seq(
        &mut self,
        _any: Self::AnyEncoder,
        len: Option<usize>,
    ) -> anyhow::Result<Self::SeqEncoder> {
        let len = len.ok_or(BinEncoderError::MissingLen)?;
        self.write_tag(TypeTag::Seq)?;
        self.write_usize(len)?;
        Ok(())
    }

    fn encode_tuple(
        &mut self,
        _any: Self::AnyEncoder,
        len: usize,
    ) -> anyhow::Result<Self::TupleEncoder> {
        self.write_tag(TypeTag::Tuple)?;
        self.write_usize(len)?;
        Ok(())
    }

    fn encode_map(
        &mut self,
        _any: Self::AnyEncoder,
        len: Option<usize>,
    ) -> anyhow::Result<Self::MapEncoder> {
        self.write_tag(TypeTag::Map)?;
        self.write_usize(len.ok_or(BinEncoderError::MissingLen)?)?;
        Ok(())
    }

    fn some_end(&mut self, _some: Self::SomeCloser) -> anyhow::Result<()> {
        Ok(())
    }

    fn tuple_encode_element(
        &mut self,
        _tuple: &mut Self::TupleEncoder,
    ) -> anyhow::Result<Self::AnyEncoder> {
        Ok(())
    }

    fn tuple_end(&mut self, _tuple: Self::TupleEncoder) -> anyhow::Result<()> {
        Ok(())
    }

    fn seq_encode_element(
        &mut self,
        _seq: &mut Self::SeqEncoder,
    ) -> anyhow::Result<Self::AnyEncoder> {
        Ok(())
    }

    fn seq_end(&mut self, _tuple: Self::SeqEncoder) -> anyhow::Result<()> {
        Ok(())
    }

    fn map_encode_element(
        &mut self,
        _map: &mut Self::MapEncoder,
    ) -> anyhow::Result<(Self::AnyEncoder, Self::ValueEncoder)> {
        Ok(((), ()))
    }

    fn map_end(&mut self, _map: Self::MapEncoder) -> anyhow::Result<()> {
        Ok(())
    }

    fn entry_encode_value(
        &mut self,
        _value: Self::ValueEncoder,
    ) -> anyhow::Result<(Self::AnyEncoder, Self::EntryCloser)> {
        Ok(((), ()))
    }

    fn entry_end(&mut self, _closer: Self::EntryCloser) -> anyhow::Result<()> {
        Ok(())
    }

    fn tuple_struct_encode_field(
        &mut self,
        _tuple: &mut Self::TupleStructEncoder,
    ) -> anyhow::Result<Self::AnyEncoder> {
        Ok(())
    }

    fn tuple_struct_end(&mut self, _map: Self::TupleStructEncoder) -> anyhow::Result<()> {
        Ok(())
    }

    fn struct_encode_field(
        &mut self,
        _map: &mut Self::StructEncoder,
        _field: &'static str,
    ) -> anyhow::Result<Self::AnyEncoder> {
        Ok(())
    }

    fn struct_end(&mut self, _map: Self::StructEncoder) -> anyhow::Result<()> {
        Ok(())
    }

    fn tuple_variant_encode_field(
        &mut self,
        _map: &mut Self::TupleVariantEncoder,
    ) -> anyhow::Result<Self::AnyEncoder> {
        Ok(())
    }

    fn tuple_variant_end(&mut self, _map: Self::TupleVariantEncoder) -> anyhow::Result<()> {
        Ok(())
    }

    fn struct_variant_encode_field(
        &mut self,
        _map: &mut Self::StructVariantEncoder,
        _key: &'static str,
    ) -> anyhow::Result<Self::AnyEncoder> {
        Ok(())
    }

    fn struct_variant_end(&mut self, _map: Self::StructVariantEncoder) -> anyhow::Result<()> {
        Ok(())
    }
}
