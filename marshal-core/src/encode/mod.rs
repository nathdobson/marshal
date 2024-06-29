use std::slice;

use crate::Primitive;

pub mod newtype;
pub mod poison;

pub trait GenEncoder: 'static {
    type SpecEncoder<'en>: SpecEncoder;
}

pub trait SpecEncoder {
    type AnyEncoder;
    type SomeCloser;
    type TupleEncoder;
    type SeqEncoder;
    type MapEncoder;
    type ValueEncoder;
    type EntryCloser;
    type TupleStructEncoder;
    type StructEncoder;
    type TupleVariantEncoder;
    type StructVariantEncoder;
    fn encode_prim(&mut self, any: Self::AnyEncoder, prim: Primitive) -> anyhow::Result<()>;
    fn encode_str(&mut self, any: Self::AnyEncoder, s: &str) -> anyhow::Result<()>;
    fn encode_bytes(&mut self, any: Self::AnyEncoder, s: &[u8]) -> anyhow::Result<()>;
    fn encode_none(&mut self, any: Self::AnyEncoder) -> anyhow::Result<()>;
    fn encode_some(
        &mut self,
        any: Self::AnyEncoder,
    ) -> anyhow::Result<(Self::AnyEncoder, Self::SomeCloser)>;
    fn encode_unit_struct(
        &mut self,
        any: Self::AnyEncoder,
        name: &'static str,
    ) -> anyhow::Result<()>;
    fn encode_tuple_struct(
        &mut self,
        any: Self::AnyEncoder,
        name: &'static str,
        len: usize,
    ) -> anyhow::Result<Self::TupleStructEncoder>;
    fn encode_struct(
        &mut self,
        any: Self::AnyEncoder,
        name: &'static str,
        fields: &'static [&'static str],
    ) -> anyhow::Result<Self::StructEncoder>;
    fn encode_unit_variant(
        &mut self,
        any: Self::AnyEncoder,
        name: &'static str,
        variants: &'static [&'static str],
        variant_index: usize,
    ) -> anyhow::Result<()>;
    fn encode_tuple_variant(
        &mut self,
        any: Self::AnyEncoder,
        name: &'static str,
        variants: &'static [&'static str],
        variant_index: usize,
        len: usize,
    ) -> anyhow::Result<Self::TupleVariantEncoder>;
    fn encode_struct_variant(
        &mut self,
        any: Self::AnyEncoder,
        name: &'static str,
        variants: &'static [&'static str],
        variant_index: usize,
        fields: &'static [&'static str],
    ) -> anyhow::Result<Self::StructVariantEncoder>;
    fn encode_seq(
        &mut self,
        any: Self::AnyEncoder,
        len: Option<usize>,
    ) -> anyhow::Result<Self::SeqEncoder>;
    fn encode_tuple(
        &mut self,
        any: Self::AnyEncoder,
        len: usize,
    ) -> anyhow::Result<Self::TupleEncoder>;
    fn encode_map(
        &mut self,
        any: Self::AnyEncoder,
        len: Option<usize>,
    ) -> anyhow::Result<Self::MapEncoder>;

    fn some_end(&mut self, some: Self::SomeCloser) -> anyhow::Result<()>;

    fn tuple_encode_element(
        &mut self,
        tuple: &mut Self::TupleEncoder,
    ) -> anyhow::Result<Self::AnyEncoder>;
    fn tuple_end(&mut self, tuple: Self::TupleEncoder) -> anyhow::Result<()>;

    fn seq_encode_element(
        &mut self,
        seq: &mut Self::SeqEncoder,
    ) -> anyhow::Result<Self::AnyEncoder>;
    fn seq_end(&mut self, tuple: Self::SeqEncoder) -> anyhow::Result<()>;

    fn map_encode_element(
        &mut self,
        map: &mut Self::MapEncoder,
    ) -> anyhow::Result<(Self::AnyEncoder, Self::ValueEncoder)>;
    fn map_end(&mut self, map: Self::MapEncoder) -> anyhow::Result<()>;

    fn entry_encode_value(
        &mut self,
        value: Self::ValueEncoder,
    ) -> anyhow::Result<(Self::AnyEncoder, Self::EntryCloser)>;
    fn entry_end(&mut self, closer: Self::EntryCloser) -> anyhow::Result<()>;

    fn tuple_struct_encode_field(
        &mut self,
        map: &mut Self::TupleStructEncoder,
    ) -> anyhow::Result<Self::AnyEncoder>;
    fn tuple_struct_end(&mut self, map: Self::TupleStructEncoder) -> anyhow::Result<()>;

    fn struct_encode_field(
        &mut self,
        map: &mut Self::StructEncoder,
        field: &'static str,
    ) -> anyhow::Result<Self::AnyEncoder>;
    fn struct_end(&mut self, map: Self::StructEncoder) -> anyhow::Result<()>;

    fn tuple_variant_encode_field(
        &mut self,
        map: &mut Self::TupleVariantEncoder,
    ) -> anyhow::Result<Self::AnyEncoder>;
    fn tuple_variant_end(&mut self, map: Self::TupleVariantEncoder) -> anyhow::Result<()>;

    fn struct_variant_encode_field(
        &mut self,
        map: &mut Self::StructVariantEncoder,
        key: &'static str,
    ) -> anyhow::Result<Self::AnyEncoder>;
    fn struct_variant_end(&mut self, map: Self::StructVariantEncoder) -> anyhow::Result<()>;
}

pub type AnyGenEncoder<'w, 'en, T> = AnyEncoder<'w, <T as GenEncoder>::SpecEncoder<'en>>;

pub struct AnyEncoder<'w, T: SpecEncoder> {
    encoder: &'w mut T,
    inner: T::AnyEncoder,
}

impl<'w, T: SpecEncoder> AnyEncoder<'w, T> {
    pub fn new(encoder: &'w mut T, inner: T::AnyEncoder) -> Self {
        AnyEncoder { encoder, inner }
    }
}

impl<'w, T: SpecEncoder> AnyEncoder<'w, T> {
    pub fn encode_prim(mut self, prim: Primitive) -> anyhow::Result<()> {
        self.encoder.encode_prim(self.inner, prim)
    }

    pub fn encode_str(mut self, s: &str) -> anyhow::Result<()> {
        self.encoder.encode_str(self.inner, s)
    }

    pub fn encode_bytes(mut self, s: &[u8]) -> anyhow::Result<()> {
        self.encoder.encode_bytes(self.inner, s)
    }

    pub fn encode_none(mut self) -> anyhow::Result<()> {
        self.encoder.encode_none(self.inner)
    }

    pub fn encode_some(mut self) -> anyhow::Result<SomeEncoder<'w, T>> {
        let (any, closer) = self.encoder.encode_some(self.inner)?;
        Ok(SomeEncoder {
            encoder: self.encoder,
            some_encoder: Some(any),
            some_closer: Some(closer),
        })
    }

    pub fn encode_unit_struct(mut self, name: &'static str) -> anyhow::Result<()> {
        self.encoder.encode_unit_struct(self.inner, name)
    }

    pub fn encode_tuple_struct(
        mut self,
        name: &'static str,
        len: usize,
    ) -> anyhow::Result<TupleStructEncoder<'w, T>> {
        let inner = self.encoder.encode_tuple_struct(self.inner, name, len)?;
        Ok(TupleStructEncoder {
            encoder: self.encoder,
            inner,
        })
    }

    pub fn encode_struct(
        mut self,
        name: &'static str,
        fields: &'static [&'static str],
    ) -> anyhow::Result<StructEncoder<'w, T>> {
        let inner = self.encoder.encode_struct(self.inner, name, fields)?;
        Ok(StructEncoder {
            encoder: self.encoder,
            fields: fields.iter(),
            inner,
        })
    }

    pub fn encode_unit_variant(
        mut self,
        name: &'static str,
        variants: &'static [&'static str],
        variant_index: usize,
    ) -> anyhow::Result<()> {
        self.encoder
            .encode_unit_variant(self.inner, name, variants, variant_index)
    }

    pub fn encode_tuple_variant(
        mut self,
        name: &'static str,
        variants: &'static [&'static str],
        variant_index: usize,
        len: usize,
    ) -> anyhow::Result<TupleVariantEncoder<'w, T>> {
        let inner =
            self.encoder
                .encode_tuple_variant(self.inner, name, variants, variant_index, len)?;
        Ok(TupleVariantEncoder {
            encoder: self.encoder,
            inner,
        })
    }

    pub fn encode_struct_variant(
        mut self,
        name: &'static str,
        variants: &'static [&'static str],
        variant_index: usize,
        fields: &'static [&'static str],
    ) -> anyhow::Result<StructVariantEncoder<'w, T>> {
        let inner = self.encoder.encode_struct_variant(
            self.inner,
            name,
            variants,
            variant_index,
            fields,
        )?;
        Ok(StructVariantEncoder {
            encoder: self.encoder,
            inner,
            fields,
        })
    }

    pub fn encode_seq(mut self, len: Option<usize>) -> anyhow::Result<SeqEncoder<'w, T>> {
        let inner = self.encoder.encode_seq(self.inner, len)?;
        Ok(SeqEncoder {
            encoder: self.encoder,
            inner,
        })
    }

    pub fn encode_tuple(mut self, len: usize) -> anyhow::Result<TupleEncoder<'w, T>> {
        let inner = self.encoder.encode_tuple(self.inner, len)?;
        Ok(TupleEncoder {
            encoder: self.encoder,
            inner,
        })
    }

    pub fn encode_map(mut self, len: Option<usize>) -> anyhow::Result<MapEncoder<'w, T>> {
        let inner = self.encoder.encode_map(self.inner, len)?;
        Ok(MapEncoder {
            encoder: self.encoder,
            inner,
        })
    }
}

pub struct SomeEncoder<'w, T: SpecEncoder> {
    encoder: &'w mut T,
    some_encoder: Option<T::AnyEncoder>,
    some_closer: Option<T::SomeCloser>,
}

impl<'w, T: SpecEncoder> SomeEncoder<'w, T> {
    pub fn encode_some(&mut self) -> anyhow::Result<AnyEncoder<'_, T>> {
        Ok(AnyEncoder {
            encoder: self.encoder,
            inner: self.some_encoder.take().unwrap(),
        })
    }

    pub fn end(mut self) -> anyhow::Result<()> {
        self.encoder.some_end(self.some_closer.take().unwrap())
    }
}

pub struct TupleEncoder<'w, T: SpecEncoder> {
    encoder: &'w mut T,
    inner: T::TupleEncoder,
}

impl<'w, T: SpecEncoder> TupleEncoder<'w, T> {
    pub fn encode_element(&mut self) -> anyhow::Result<AnyEncoder<'_, T>> {
        let inner = self.encoder.tuple_encode_element(&mut self.inner)?;
        Ok(AnyEncoder {
            encoder: self.encoder,
            inner,
        })
    }

    pub fn end(self) -> anyhow::Result<()> {
        self.encoder.tuple_end(self.inner)
    }
}

pub struct SeqEncoder<'w, T: SpecEncoder> {
    encoder: &'w mut T,
    inner: T::SeqEncoder,
}

impl<'w, T: SpecEncoder> SeqEncoder<'w, T> {
    pub fn encode_element(&mut self) -> anyhow::Result<AnyEncoder<'_, T>> {
        let inner = self.encoder.seq_encode_element(&mut self.inner)?;
        Ok(AnyEncoder {
            encoder: self.encoder,
            inner,
        })
    }

    pub fn end(self) -> anyhow::Result<()> {
        self.encoder.seq_end(self.inner)
    }
}

pub struct MapEncoder<'w, T: SpecEncoder> {
    encoder: &'w mut T,
    inner: T::MapEncoder,
}

impl<'w, T: SpecEncoder> MapEncoder<'w, T> {
    pub fn encode_entry(&mut self) -> anyhow::Result<EntryEncoder<'_, T>> {
        let (key, value) = self.encoder.map_encode_element(&mut self.inner)?;
        Ok(EntryEncoder {
            encoder: self.encoder,
            key: Some(key),
            value: Some(value),
            closer: None,
        })
    }

    pub fn end(self) -> anyhow::Result<()> {
        self.encoder.map_end(self.inner)
    }
}

pub struct EntryEncoder<'w, T: SpecEncoder> {
    encoder: &'w mut T,
    key: Option<T::AnyEncoder>,
    value: Option<T::ValueEncoder>,
    closer: Option<T::EntryCloser>,
}

impl<'w, T: SpecEncoder> EntryEncoder<'w, T> {
    pub fn encode_key(&mut self) -> anyhow::Result<AnyEncoder<'_, T>> {
        Ok(AnyEncoder {
            encoder: self.encoder,
            inner: self.key.take().unwrap(),
        })
    }

    pub fn encode_value(&mut self) -> anyhow::Result<AnyEncoder<'_, T>> {
        let (any, closer) = self
            .encoder
            .entry_encode_value(self.value.take().unwrap())?;
        self.closer = Some(closer);
        Ok(AnyEncoder {
            encoder: self.encoder,
            inner: any,
        })
    }

    pub fn end(self) -> anyhow::Result<()> {
        self.encoder.entry_end(self.closer.unwrap())
    }
}

pub struct TupleStructEncoder<'w, T: SpecEncoder> {
    encoder: &'w mut T,
    inner: T::TupleStructEncoder,
}

impl<'w, T: SpecEncoder> TupleStructEncoder<'w, T> {
    pub fn encode_field(&mut self) -> anyhow::Result<AnyEncoder<'_, T>> {
        let inner = self.encoder.tuple_struct_encode_field(&mut self.inner)?;
        Ok(AnyEncoder {
            encoder: self.encoder,
            inner,
        })
    }

    pub fn end(self) -> anyhow::Result<()> {
        self.encoder.tuple_struct_end(self.inner)
    }
}

pub struct StructEncoder<'w, T: SpecEncoder> {
    encoder: &'w mut T,
    fields: slice::Iter<'static, &'static str>,
    inner: T::StructEncoder,
}

impl<'w, T: SpecEncoder> StructEncoder<'w, T> {
    pub fn encode_field(&mut self) -> anyhow::Result<AnyEncoder<'_, T>> {
        let inner = self
            .encoder
            .struct_encode_field(&mut self.inner, self.fields.next().unwrap())?;
        Ok(AnyEncoder {
            encoder: self.encoder,
            inner,
        })
    }

    pub fn end(self) -> anyhow::Result<()> {
        self.encoder.struct_end(self.inner)
    }
}

pub struct TupleVariantEncoder<'w, T: SpecEncoder> {
    encoder: &'w mut T,
    inner: T::TupleVariantEncoder,
}

impl<'w, T: SpecEncoder> TupleVariantEncoder<'w, T> {
    pub fn encode_field(&mut self) -> anyhow::Result<AnyEncoder<'_, T>> {
        let inner = self.encoder.tuple_variant_encode_field(&mut self.inner)?;
        Ok(AnyEncoder {
            encoder: self.encoder,
            inner,
        })
    }

    pub fn end(self) -> anyhow::Result<()> {
        self.encoder.tuple_variant_end(self.inner)
    }
}

pub struct StructVariantEncoder<'w, T: SpecEncoder> {
    encoder: &'w mut T,
    inner: T::StructVariantEncoder,
    fields: &'static [&'static str],
}

impl<'w, T: SpecEncoder> StructVariantEncoder<'w, T> {
    pub fn encode_field(&mut self) -> anyhow::Result<AnyEncoder<'_, T>> {
        let inner = self
            .encoder
            .struct_variant_encode_field(&mut self.inner, self.fields.take_first().unwrap())?;
        Ok(AnyEncoder {
            encoder: self.encoder,
            inner,
        })
    }

    pub fn end(self) -> anyhow::Result<()> {
        self.encoder.struct_variant_end(self.inner)
    }
}
