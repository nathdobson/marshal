pub mod full;

use by_address::ByAddress;
use marshal_core::write::simple::SimpleWriter;
use marshal_core::Primitive;
use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::fmt::{Debug, Display, Formatter};

use crate::to_from_vu128::{Array, ToFromVu128};
use crate::{TypeTag, VU128_MAX_PADDING};

pub struct BinWriterSchema {
    enum_def_indexes: HashMap<ByAddress<&'static [&'static str]>, usize>,
}

impl BinWriterSchema {
    pub fn new() -> Self {
        BinWriterSchema {
            enum_def_indexes: HashMap::new(),
        }
    }
}

#[derive(Debug)]
pub enum BinWriterError {
    MissingLen,
}

impl Display for BinWriterError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        Debug::fmt(self, f)
    }
}

impl std::error::Error for BinWriterError {}

pub struct SimpleBinWriter<'s> {
    output: Vec<u8>,
    schema: &'s mut BinWriterSchema,
}

impl<'s> SimpleBinWriter<'s> {
    pub fn new(schema: &mut BinWriterSchema) -> SimpleBinWriter {
        SimpleBinWriter {
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
    pub fn write_bytes(&mut self, value: &[u8]) -> anyhow::Result<()> {
        self.write_usize(value.len())?;
        self.write_raw(value)?;
        Ok(())
    }

    pub fn write_str(&mut self, value: &str) -> anyhow::Result<()> {
        self.write_bytes(value.as_bytes())
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
            self.write_str(x)?;
        }
        let new_index = self.schema.enum_def_indexes.len();
        self.schema
            .enum_def_indexes
            .insert(ByAddress(variants), new_index);
        Ok(new_index)
    }
}

impl<'s> SimpleWriter for SimpleBinWriter<'s> {
    type AnyWriter = ();
    type SomeCloser = ();
    type TupleWriter = ();
    type SeqWriter = ();
    type MapWriter = ();
    type ValueWriter = ();
    type EntryCloser = ();
    type TupleStructWriter = ();
    type StructWriter = ();
    type TupleVariantWriter = ();
    type StructVariantWriter = ();

    fn write_prim(&mut self, any: Self::AnyWriter, prim: Primitive) -> anyhow::Result<()> {
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

    fn write_str(&mut self, any: Self::AnyWriter, s: &str) -> anyhow::Result<()> {
        self.write_tag(TypeTag::String)?;
        self.write_str(s)?;
        Ok(())
    }

    fn write_bytes(&mut self, any: Self::AnyWriter, s: &[u8]) -> anyhow::Result<()> {
        todo!()
    }

    fn write_none(&mut self, any: Self::AnyWriter) -> anyhow::Result<()> {
        todo!()
    }

    fn write_some(
        &mut self,
        any: Self::AnyWriter,
    ) -> anyhow::Result<(Self::AnyWriter, Self::SomeCloser)> {
        todo!()
    }

    fn write_unit_struct(
        &mut self,
        any: Self::AnyWriter,
        name: &'static str,
    ) -> anyhow::Result<()> {
        self.write_tag(TypeTag::UnitStruct)
    }

    fn write_tuple_struct(
        &mut self,
        any: Self::AnyWriter,
        name: &'static str,
        len: usize,
    ) -> anyhow::Result<Self::TupleStructWriter> {
        self.write_tag(TypeTag::TupleStruct)?;
        self.write_usize(len)?;
        Ok(())
    }

    fn write_struct(
        &mut self,
        any: Self::AnyWriter,
        name: &'static str,
        fields: &'static [&'static str],
    ) -> anyhow::Result<Self::StructWriter> {
        let def = self.get_or_write_enum_def(fields)?;
        self.write_tag(TypeTag::Struct)?;
        self.write_usize(def)?;
        Ok(())
    }

    fn write_unit_variant(
        &mut self,
        any: Self::AnyWriter,
        name: &'static str,
        variants: &'static [&'static str],
        variant_index: u32,
    ) -> anyhow::Result<()> {
        let enum_def = self.get_or_write_enum_def(variants)?;
        self.write_tag(TypeTag::Enum)?;
        self.write_usize(enum_def)?;
        self.write_vu128(variant_index)?;
        self.write_tag(TypeTag::Unit)?;
        Ok(())
    }

    fn write_tuple_variant(
        &mut self,
        any: Self::AnyWriter,
        name: &'static str,
        variants: &'static [&'static str],
        variant_index: u32,
        len: usize,
    ) -> anyhow::Result<Self::TupleVariantWriter> {
        todo!()
    }

    fn write_struct_variant(
        &mut self,
        any: Self::AnyWriter,
        name: &'static str,
        variants: &'static [&'static str],
        variant_index: u32,
        len: usize,
    ) -> anyhow::Result<Self::StructVariantWriter> {
        todo!()
    }

    fn write_seq(
        &mut self,
        any: Self::AnyWriter,
        len: Option<usize>,
    ) -> anyhow::Result<Self::SeqWriter> {
        let len = len.ok_or(BinWriterError::MissingLen)?;
        self.write_tag(TypeTag::Seq)?;
        self.write_usize(len)?;
        Ok(())
    }

    fn write_tuple(
        &mut self,
        any: Self::AnyWriter,
        len: usize,
    ) -> anyhow::Result<Self::TupleWriter> {
        self.write_tag(TypeTag::Tuple)?;
        self.write_vu128(len as u32)?;
        Ok(())
    }

    fn write_map(
        &mut self,
        any: Self::AnyWriter,
        len: Option<usize>,
    ) -> anyhow::Result<Self::MapWriter> {
        todo!()
    }

    fn some_end(&mut self, some: Self::SomeCloser) -> anyhow::Result<()> {
        todo!()
    }

    fn tuple_write_element(
        &mut self,
        tuple: &mut Self::TupleWriter,
    ) -> anyhow::Result<Self::AnyWriter> {
        Ok(())
    }

    fn tuple_end(&mut self, tuple: Self::TupleWriter) -> anyhow::Result<()> {
        Ok(())
    }

    fn seq_write_element(&mut self, seq: &mut Self::SeqWriter) -> anyhow::Result<Self::AnyWriter> {
        Ok(())
    }

    fn seq_end(&mut self, tuple: Self::SeqWriter) -> anyhow::Result<()> {
        Ok(())
    }

    fn map_write_element(
        &mut self,
        map: &mut Self::MapWriter,
    ) -> anyhow::Result<(Self::AnyWriter, Self::ValueWriter)> {
        todo!()
    }

    fn map_end(&mut self, map: Self::MapWriter) -> anyhow::Result<()> {
        todo!()
    }

    fn entry_write_value(
        &mut self,
        value: Self::ValueWriter,
    ) -> anyhow::Result<(Self::AnyWriter, Self::EntryCloser)> {
        todo!()
    }

    fn entry_end(&mut self, closer: Self::EntryCloser) -> anyhow::Result<()> {
        todo!()
    }

    fn tuple_struct_write_field(
        &mut self,
        tuple: &mut Self::TupleStructWriter,
    ) -> anyhow::Result<Self::AnyWriter> {
        Ok(())
    }

    fn tuple_struct_end(&mut self, map: Self::TupleStructWriter) -> anyhow::Result<()> {
        Ok(())
    }

    fn struct_write_field(
        &mut self,
        map: &mut Self::StructWriter,
        field: &'static str,
    ) -> anyhow::Result<Self::AnyWriter> {
        Ok(())
    }

    fn struct_end(&mut self, map: Self::StructWriter) -> anyhow::Result<()> {
        Ok(())
    }

    fn tuple_variant_write_field(
        &mut self,
        map: &mut Self::TupleVariantWriter,
    ) -> anyhow::Result<Self::AnyWriter> {
        todo!()
    }

    fn tuple_variant_end(&mut self, map: Self::TupleVariantWriter) -> anyhow::Result<()> {
        todo!()
    }

    fn struct_variant_write_field(
        &mut self,
        map: &mut Self::StructVariantWriter,
        key: &'static str,
    ) -> anyhow::Result<Self::AnyWriter> {
        todo!()
    }

    fn struct_variant_end(&mut self, map: Self::StructVariantWriter) -> anyhow::Result<()> {
        todo!()
    }
}
