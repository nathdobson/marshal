pub mod full;

use crate::DiscriminantWidth;
use anyhow::anyhow;
use marshal::encode::SpecEncoder;
use marshal::Primitive;
use marshal_vu128::{WriteVu128, VU128_PADDING};

pub struct SimpleFixedSpecEncoder {
    output: Vec<u8>,
}

impl SimpleFixedSpecEncoder {
    pub fn new() -> Self {
        SimpleFixedSpecEncoder { output: vec![] }
    }
    #[inline]
    pub fn encode_discriminant(&mut self, index: usize, max: usize) {
        match DiscriminantWidth::from_max(max) {
            DiscriminantWidth::U8 => self.output.write_vu128(index as u8),
            DiscriminantWidth::U16 => self.output.write_vu128(index as u16),
            DiscriminantWidth::U32 => self.output.write_vu128(index as u32),
            DiscriminantWidth::U64 => self.output.write_vu128(index as u64),
        }
    }
    #[inline]
    pub fn end(mut self) -> anyhow::Result<Vec<u8>> {
        self.output.resize(self.output.len() + VU128_PADDING, 0);
        Ok(self.output)
    }
}

impl SpecEncoder for SimpleFixedSpecEncoder {
    type AnySpecEncoder = ();
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

    #[inline]
    fn encode_prim(&mut self, _: Self::AnySpecEncoder, prim: Primitive) -> anyhow::Result<()> {
        match prim {
            Primitive::Unit => {}
            Primitive::Bool(x) => self.output.write_vu128(x),
            Primitive::I8(x) => self.output.write_vu128(x),
            Primitive::I16(x) => self.output.write_vu128(x),
            Primitive::I32(x) => self.output.write_vu128(x),
            Primitive::I64(x) => self.output.write_vu128(x),
            Primitive::I128(x) => self.output.write_vu128(x),
            Primitive::U8(x) => self.output.write_vu128(x),
            Primitive::U16(x) => self.output.write_vu128(x),
            Primitive::U32(x) => self.output.write_vu128(x),
            Primitive::U64(x) => self.output.write_vu128(x),
            Primitive::U128(x) => self.output.write_vu128(x),
            Primitive::F32(x) => self.output.write_vu128(x),
            Primitive::F64(x) => self.output.write_vu128(x),
            Primitive::Char(x) => self.output.write_vu128(x as u32),
        }
        Ok(())
    }

    #[inline]
    fn encode_str(&mut self, _: Self::AnySpecEncoder, s: &str) -> anyhow::Result<()> {
        self.output.write_vu128(s.len() as u64);
        self.output.extend_from_slice(s.as_bytes());
        Ok(())
    }

    #[inline]
    fn encode_bytes(&mut self, _: Self::AnySpecEncoder, s: &[u8]) -> anyhow::Result<()> {
        self.output.write_vu128(s.len() as u64);
        self.output.extend_from_slice(s);
        Ok(())
    }

    #[inline]
    fn encode_none(&mut self, _: Self::AnySpecEncoder) -> anyhow::Result<()> {
        self.output.write_vu128(false);
        Ok(())
    }

    #[inline]
    fn encode_some(
        &mut self,
        _: Self::AnySpecEncoder,
    ) -> anyhow::Result<(Self::AnySpecEncoder, Self::SomeCloser)> {
        self.output.write_vu128(true);
        Ok(((), ()))
    }

    #[inline]
    fn encode_unit_struct(
        &mut self,
        _: Self::AnySpecEncoder,
        _: &'static str,
    ) -> anyhow::Result<()> {
        Ok(())
    }

    #[inline]
    fn encode_tuple_struct(
        &mut self,
        _: Self::AnySpecEncoder,
        _: &'static str,
        _: usize,
    ) -> anyhow::Result<Self::TupleStructEncoder> {
        Ok(())
    }

    #[inline]
    fn encode_struct(
        &mut self,
        _: Self::AnySpecEncoder,
        _: &'static str,
        _: &'static [&'static str],
    ) -> anyhow::Result<Self::StructEncoder> {
        Ok(())
    }

    #[inline]
    fn encode_unit_variant(
        &mut self,
        _: Self::AnySpecEncoder,
        _: &'static str,
        _: &'static [&'static str],
        _: usize,
    ) -> anyhow::Result<()> {
        Ok(())
    }

    #[inline]
    fn encode_tuple_variant(
        &mut self,
        _: Self::AnySpecEncoder,
        _: &'static str,
        variants: &'static [&'static str],
        variant_index: usize,
        _: usize,
    ) -> anyhow::Result<Self::TupleVariantEncoder> {
        self.encode_discriminant(variant_index, variants.len());
        Ok(())
    }

    #[inline]
    fn encode_struct_variant(
        &mut self,
        _: Self::AnySpecEncoder,
        _: &'static str,
        variants: &'static [&'static str],
        variant_index: usize,
        _: &'static [&'static str],
    ) -> anyhow::Result<Self::StructVariantEncoder> {
        self.encode_discriminant(variant_index, variants.len());
        Ok(())
    }

    #[inline]
    fn encode_seq(
        &mut self,
        _: Self::AnySpecEncoder,
        len: Option<usize>,
    ) -> anyhow::Result<Self::SeqEncoder> {
        self.output
            .write_vu128(len.ok_or_else(|| anyhow!("missing seq length"))? as u64);
        Ok(())
    }

    #[inline]
    fn encode_tuple(
        &mut self,
        _: Self::AnySpecEncoder,
        _: usize,
    ) -> anyhow::Result<Self::TupleEncoder> {
        Ok(())
    }

    #[inline]
    fn encode_map(
        &mut self,
        _: Self::AnySpecEncoder,
        len: Option<usize>,
    ) -> anyhow::Result<Self::MapEncoder> {
        self.output
            .write_vu128(len.ok_or_else(|| anyhow!("missing map length"))? as u64);
        Ok(())
    }

    #[inline]
    fn some_end(&mut self, _: Self::SomeCloser) -> anyhow::Result<()> {
        Ok(())
    }

    #[inline]
    fn tuple_encode_element(
        &mut self,
        _: &mut Self::TupleEncoder,
    ) -> anyhow::Result<Self::AnySpecEncoder> {
        Ok(())
    }

    #[inline]
    fn tuple_end(&mut self, _: Self::TupleEncoder) -> anyhow::Result<()> {
        Ok(())
    }

    #[inline]
    fn seq_encode_element(
        &mut self,
        _: &mut Self::SeqEncoder,
    ) -> anyhow::Result<Self::AnySpecEncoder> {
        Ok(())
    }

    #[inline]
    fn seq_end(&mut self, _: Self::SeqEncoder) -> anyhow::Result<()> {
        Ok(())
    }

    #[inline]
    fn map_encode_element(
        &mut self,
        _: &mut Self::MapEncoder,
    ) -> anyhow::Result<(Self::AnySpecEncoder, Self::ValueEncoder)> {
        Ok(((), ()))
    }

    #[inline]
    fn map_end(&mut self, _: Self::MapEncoder) -> anyhow::Result<()> {
        Ok(())
    }

    #[inline]
    fn entry_encode_value(
        &mut self,
        _: Self::ValueEncoder,
    ) -> anyhow::Result<(Self::AnySpecEncoder, Self::EntryCloser)> {
        Ok(((), ()))
    }

    #[inline]
    fn entry_end(&mut self, _: Self::EntryCloser) -> anyhow::Result<()> {
        Ok(())
    }

    #[inline]
    fn tuple_struct_encode_field(
        &mut self,
        _: &mut Self::TupleStructEncoder,
    ) -> anyhow::Result<Self::AnySpecEncoder> {
        Ok(())
    }

    #[inline]
    fn tuple_struct_end(&mut self, _: Self::TupleStructEncoder) -> anyhow::Result<()> {
        Ok(())
    }

    #[inline]
    fn struct_encode_field(
        &mut self,
        _: &mut Self::StructEncoder,
        _: &'static str,
    ) -> anyhow::Result<Self::AnySpecEncoder> {
        Ok(())
    }

    #[inline]
    fn struct_end(&mut self, _: Self::StructEncoder) -> anyhow::Result<()> {
        Ok(())
    }

    #[inline]
    fn tuple_variant_encode_field(
        &mut self,
        _: &mut Self::TupleVariantEncoder,
    ) -> anyhow::Result<Self::AnySpecEncoder> {
        Ok(())
    }

    #[inline]
    fn tuple_variant_end(&mut self, _: Self::TupleVariantEncoder) -> anyhow::Result<()> {
        Ok(())
    }

    #[inline]
    fn struct_variant_encode_field(
        &mut self,
        _: &mut Self::StructVariantEncoder,
        _: &'static str,
    ) -> anyhow::Result<Self::AnySpecEncoder> {
        Ok(())
    }

    #[inline]
    fn struct_variant_end(&mut self, _: Self::StructVariantEncoder) -> anyhow::Result<()> {
        Ok(())
    }

    #[inline]
    fn is_human_readable(&self) -> bool {
        false
    }
}
