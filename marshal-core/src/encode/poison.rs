use std::fmt::{Display, Formatter};

use crate::encode::Encoder;
use crate::Primitive;

pub struct PoisonEncoder<E> {
    inner: E,
    depth: usize,
}

#[derive(Debug)]
pub enum PoisonError {
    UnexpectedEncodeState,
}
impl Display for PoisonError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        std::fmt::Debug::fmt(self, f)
    }
}
impl std::error::Error for PoisonError {}

impl<E: Encoder> PoisonEncoder<E> {
    pub fn new(inner: E) -> Self {
        PoisonEncoder { inner, depth: 0 }
    }
    pub fn start(&mut self, inner: E::AnyEncoder) -> <Self as Encoder>::AnyEncoder {
        self.push(inner)
    }
    pub fn end(self) -> anyhow::Result<E> {
        if self.depth == 0 {
            Ok(self.inner)
        } else {
            Err(PoisonError::UnexpectedEncodeState.into())
        }
    }
    fn push<T>(&mut self, inner: T) -> PoisonWrapper<T> {
        self.depth += 1;
        PoisonWrapper {
            depth: self.depth,
            inner,
        }
    }
    fn pop<T>(&mut self, wrapper: PoisonWrapper<T>) -> anyhow::Result<T> {
        if wrapper.depth == self.depth {
            self.depth -= 1;
            Ok(wrapper.inner)
        } else {
            Err(PoisonError::UnexpectedEncodeState.into())
        }
    }
    fn peek<'a, T>(&self, wrapper: &'a mut PoisonWrapper<T>) -> anyhow::Result<&'a mut T> {
        if wrapper.depth == self.depth {
            Ok(&mut wrapper.inner)
        } else {
            Err(PoisonError::UnexpectedEncodeState.into())
        }
    }
}

pub struct PoisonWrapper<T> {
    depth: usize,
    inner: T,
}

impl<E: Encoder> Encoder for PoisonEncoder<E> {
    type AnyEncoder = PoisonWrapper<E::AnyEncoder>;
    type SomeCloser = PoisonWrapper<E::SomeCloser>;
    type TupleEncoder = PoisonWrapper<E::TupleEncoder>;
    type SeqEncoder = PoisonWrapper<E::SeqEncoder>;
    type MapEncoder = PoisonWrapper<E::MapEncoder>;
    type ValueEncoder = PoisonWrapper<E::ValueEncoder>;
    type EntryCloser = PoisonWrapper<E::EntryCloser>;
    type TupleStructEncoder = PoisonWrapper<E::TupleStructEncoder>;
    type StructEncoder = PoisonWrapper<E::StructEncoder>;
    type TupleVariantEncoder = PoisonWrapper<E::TupleVariantEncoder>;
    type StructVariantEncoder = PoisonWrapper<E::StructVariantEncoder>;

    fn encode_prim(&mut self, any: Self::AnyEncoder, prim: Primitive) -> anyhow::Result<()> {
        let any = self.pop(any)?;
        self.inner.encode_prim(any, prim)
    }

    fn encode_str(&mut self, any: Self::AnyEncoder, s: &str) -> anyhow::Result<()> {
        let any = self.pop(any)?;
        self.inner.encode_str(any, s)
    }

    fn encode_bytes(&mut self, any: Self::AnyEncoder, s: &[u8]) -> anyhow::Result<()> {
        let any = self.pop(any)?;
        self.inner.encode_bytes(any, s)
    }

    fn encode_none(&mut self, any: Self::AnyEncoder) -> anyhow::Result<()> {
        let any = self.pop(any)?;
        self.inner.encode_none(any)
    }

    fn encode_some(
        &mut self,
        any: Self::AnyEncoder,
    ) -> anyhow::Result<(Self::AnyEncoder, Self::SomeCloser)> {
        let any = self.pop(any)?;
        let (some_encoder, some_closer) = self.inner.encode_some(any)?;
        let some_closer = self.push(some_closer);
        let some_encoder = self.push(some_encoder);
        Ok((some_encoder, some_closer))
    }

    fn encode_unit_struct(
        &mut self,
        any: Self::AnyEncoder,
        name: &'static str,
    ) -> anyhow::Result<()> {
        let any = self.pop(any)?;
        self.inner.encode_unit_struct(any, name)
    }

    fn encode_tuple_struct(
        &mut self,
        any: Self::AnyEncoder,
        name: &'static str,
        len: usize,
    ) -> anyhow::Result<Self::TupleStructEncoder> {
        let any = self.pop(any)?;
        let tuple_struct_encoder = self.inner.encode_tuple_struct(any, name, len)?;
        Ok(self.push(tuple_struct_encoder))
    }

    fn encode_struct(
        &mut self,
        any: Self::AnyEncoder,
        name: &'static str,
        fields: &'static [&'static str],
    ) -> anyhow::Result<Self::StructEncoder> {
        let any = self.pop(any)?;
        let struct_encoder = self.inner.encode_struct(any, name, fields)?;
        Ok(self.push(struct_encoder))
    }

    fn encode_unit_variant(
        &mut self,
        any: Self::AnyEncoder,
        name: &'static str,
        variants: &'static [&'static str],
        variant_index: usize,
    ) -> anyhow::Result<()> {
        let any = self.pop(any)?;
        self.inner
            .encode_unit_variant(any, name, variants, variant_index)?;
        Ok(())
    }

    fn encode_tuple_variant(
        &mut self,
        any: Self::AnyEncoder,
        name: &'static str,
        variants: &'static [&'static str],
        variant_index: usize,
        len: usize,
    ) -> anyhow::Result<Self::TupleVariantEncoder> {
        let any = self.pop(any)?;
        let encoder = self
            .inner
            .encode_tuple_variant(any, name, variants, variant_index, len)?;
        Ok(self.push(encoder))
    }

    fn encode_struct_variant(
        &mut self,
        any: Self::AnyEncoder,
        name: &'static str,
        variants: &'static [&'static str],
        variant_index: usize,
        fields: &'static [&'static str],
    ) -> anyhow::Result<Self::StructVariantEncoder> {
        let any = self.pop(any)?;
        let encoder =
            self.inner
                .encode_struct_variant(any, name, variants, variant_index, fields)?;
        Ok(self.push(encoder))
    }

    fn encode_seq(
        &mut self,
        any: Self::AnyEncoder,
        len: Option<usize>,
    ) -> anyhow::Result<Self::SeqEncoder> {
        let any = self.pop(any)?;
        let encoder = self.inner.encode_seq(any, len)?;
        Ok(self.push(encoder))
    }

    fn encode_tuple(
        &mut self,
        any: Self::AnyEncoder,
        len: usize,
    ) -> anyhow::Result<Self::TupleEncoder> {
        let any = self.pop(any)?;
        let encoder = self.inner.encode_tuple(any, len)?;
        Ok(self.push(encoder))
    }

    fn encode_map(
        &mut self,
        any: Self::AnyEncoder,
        len: Option<usize>,
    ) -> anyhow::Result<Self::MapEncoder> {
        let any = self.pop(any)?;
        let encoder = self.inner.encode_map(any, len)?;
        Ok(self.push(encoder))
    }

    fn some_end(&mut self, some: Self::SomeCloser) -> anyhow::Result<()> {
        let some = self.pop(some)?;
        self.inner.some_end(some)
    }

    fn tuple_encode_element(
        &mut self,
        tuple: &mut Self::TupleEncoder,
    ) -> anyhow::Result<Self::AnyEncoder> {
        let tuple = self.peek(tuple)?;
        let encoder = self.inner.tuple_encode_element(tuple)?;
        Ok(self.push(encoder))
    }

    fn tuple_end(&mut self, tuple: Self::TupleEncoder) -> anyhow::Result<()> {
        let tuple = self.pop(tuple)?;
        self.inner.tuple_end(tuple)
    }

    fn seq_encode_element(
        &mut self,
        seq: &mut Self::SeqEncoder,
    ) -> anyhow::Result<Self::AnyEncoder> {
        let seq = self.peek(seq)?;
        let encoder = self.inner.seq_encode_element(seq)?;
        Ok(self.push(encoder))
    }

    fn seq_end(&mut self, seq: Self::SeqEncoder) -> anyhow::Result<()> {
        let seq = self.pop(seq)?;
        self.inner.seq_end(seq)
    }

    fn map_encode_element(
        &mut self,
        map: &mut Self::MapEncoder,
    ) -> anyhow::Result<(Self::AnyEncoder, Self::ValueEncoder)> {
        let map = self.peek(map)?;
        let (key, value) = self.inner.map_encode_element(map)?;
        let value = self.push(value);
        let key = self.push(key);
        Ok((key, value))
    }

    fn map_end(&mut self, map: Self::MapEncoder) -> anyhow::Result<()> {
        let map = self.pop(map)?;
        self.inner.map_end(map)?;
        Ok(())
    }

    fn entry_encode_value(
        &mut self,
        value: Self::ValueEncoder,
    ) -> anyhow::Result<(Self::AnyEncoder, Self::EntryCloser)> {
        let value = self.pop(value)?;
        let (value, closer) = self.inner.entry_encode_value(value)?;
        let closer = self.push(closer);
        let value = self.push(value);
        Ok((value, closer))
    }

    fn entry_end(&mut self, closer: Self::EntryCloser) -> anyhow::Result<()> {
        let closer = self.pop(closer)?;
        self.inner.entry_end(closer)?;
        Ok(())
    }

    fn tuple_struct_encode_field(
        &mut self,
        struc: &mut Self::TupleStructEncoder,
    ) -> anyhow::Result<Self::AnyEncoder> {
        let struc = self.peek(struc)?;
        let encoder = self.inner.tuple_struct_encode_field(struc)?;
        Ok(self.push(encoder))
    }

    fn tuple_struct_end(&mut self, struc: Self::TupleStructEncoder) -> anyhow::Result<()> {
        let struc = self.pop(struc)?;
        self.inner.tuple_struct_end(struc)?;
        Ok(())
    }

    fn struct_encode_field(
        &mut self,
        struc: &mut Self::StructEncoder,
        field: &'static str,
    ) -> anyhow::Result<Self::AnyEncoder> {
        let struc = self.peek(struc)?;
        let encoder = self.inner.struct_encode_field(struc, field)?;
        Ok(self.push(encoder))
    }

    fn struct_end(&mut self, struc: Self::StructEncoder) -> anyhow::Result<()> {
        let struc = self.pop(struc)?;
        self.inner.struct_end(struc)?;
        Ok(())
    }

    fn tuple_variant_encode_field(
        &mut self,
        variant: &mut Self::TupleVariantEncoder,
    ) -> anyhow::Result<Self::AnyEncoder> {
        let variant = self.peek(variant)?;
        let encoder = self.inner.tuple_variant_encode_field(variant)?;
        Ok(self.push(encoder))
    }

    fn tuple_variant_end(&mut self, variant: Self::TupleVariantEncoder) -> anyhow::Result<()> {
        let variant = self.pop(variant)?;
        self.inner.tuple_variant_end(variant)?;
        Ok(())
    }

    fn struct_variant_encode_field(
        &mut self,
        variant: &mut Self::StructVariantEncoder,
        field: &'static str,
    ) -> anyhow::Result<Self::AnyEncoder> {
        let variant = self.peek(variant)?;
        let encoder = self.inner.struct_variant_encode_field(variant, field)?;
        Ok(self.push(encoder))
    }

    fn struct_variant_end(&mut self, variant: Self::StructVariantEncoder) -> anyhow::Result<()> {
        let variant = self.pop(variant)?;
        self.inner.struct_variant_end(variant)?;
        Ok(())
    }
}
