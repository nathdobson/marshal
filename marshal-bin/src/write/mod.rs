use marshal_core::Primitive;
use marshal_core::write::simple::SimpleWriter;

use crate::to_from_vu128::{Array, ToFromVu128};

pub struct BinSchema {}

pub struct BinWriter<'s> {
    output: Vec<u8>,
    schema: &'s mut BinSchema,
}

impl<'s> BinWriter<'s> {
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
}

impl<'s> SimpleWriter for BinWriter<'s> {
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
        todo!()
    }

    fn write_str(&mut self, any: Self::AnyWriter, s: &str) -> anyhow::Result<()> {
        todo!()
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
        todo!()
    }

    fn write_tuple_struct(
        &mut self,
        any: Self::AnyWriter,
        name: &'static str,
        len: usize,
    ) -> anyhow::Result<Self::TupleStructWriter> {
        todo!()
    }

    fn write_struct(
        &mut self,
        any: Self::AnyWriter,
        name: &'static str,
        len: usize,
    ) -> anyhow::Result<Self::StructWriter> {
        todo!()
    }

    fn write_unit_variant(
        &mut self,
        any: Self::AnyWriter,
        name: &'static str,
        variant_index: u32,
        variant: &'static str,
    ) -> anyhow::Result<()> {
        todo!()
    }

    fn write_tuple_variant(
        &mut self,
        any: Self::AnyWriter,
        name: &'static str,
        variant_index: u32,
        variant: &'static str,
        len: usize,
    ) -> anyhow::Result<Self::TupleVariantWriter> {
        todo!()
    }

    fn write_struct_variant(
        &mut self,
        any: Self::AnyWriter,
        name: &'static str,
        variant_index: u32,
        variant: &'static str,
        len: usize,
    ) -> anyhow::Result<Self::StructVariantWriter> {
        todo!()
    }

    fn write_seq(
        &mut self,
        any: Self::AnyWriter,
        len: Option<usize>,
    ) -> anyhow::Result<Self::SeqWriter> {
        todo!()
    }

    fn write_tuple(
        &mut self,
        any: Self::AnyWriter,
        len: usize,
    ) -> anyhow::Result<Self::TupleWriter> {
        todo!()
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
        todo!()
    }

    fn tuple_end(&mut self, tuple: Self::TupleWriter) -> anyhow::Result<()> {
        todo!()
    }

    fn seq_write_element(&mut self, seq: &mut Self::SeqWriter) -> anyhow::Result<Self::AnyWriter> {
        todo!()
    }

    fn seq_end(&mut self, tuple: Self::SeqWriter) -> anyhow::Result<()> {
        todo!()
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
        map: &mut Self::TupleStructWriter,
    ) -> anyhow::Result<Self::AnyWriter> {
        todo!()
    }

    fn tuple_struct_end(&mut self, map: Self::TupleStructWriter) -> anyhow::Result<()> {
        todo!()
    }

    fn struct_write_field(
        &mut self,
        map: &mut Self::StructWriter,
        key: &'static str,
    ) -> anyhow::Result<Self::AnyWriter> {
        todo!()
    }

    fn struct_end(&mut self, map: Self::StructWriter) -> anyhow::Result<()> {
        todo!()
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
