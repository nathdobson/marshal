use std::marker::PhantomData;
use std::slice;

use crate::encode::{
    AnyEncoder, Encoder, EntryEncoder, MapEncoder, SeqEncoder, SomeEncoder, StructEncoder,
    StructVariantEncoder, TupleEncoder, TupleStructEncoder, TupleVariantEncoder,
};
use crate::Primitive;

pub trait SimpleEncoder {
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
    fn encode_unit_struct(&mut self, any: Self::AnyEncoder, name: &'static str)
                          -> anyhow::Result<()>;
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

    fn seq_encode_element(&mut self, seq: &mut Self::SeqEncoder) -> anyhow::Result<Self::AnyEncoder>;
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

pub struct SimpleEncoderAdapter<T>(PhantomData<T>);

pub struct SimpleAnyEncoder<'w, T: SimpleEncoder> {
    encoder: &'w mut T,
    inner: T::AnyEncoder,
}

impl<'w, T: SimpleEncoder> SimpleAnyEncoder<'w, T> {
    pub fn new(encoder: &'w mut T, inner: T::AnyEncoder) -> Self {
        SimpleAnyEncoder { encoder, inner }
    }
}

impl<'w, T: SimpleEncoder> AnyEncoder<'w, SimpleEncoderAdapter<T>> for SimpleAnyEncoder<'w, T> {
    fn encode_prim(mut self, prim: Primitive) -> anyhow::Result<()> {
        self.encoder.encode_prim(self.inner, prim)
    }

    fn encode_str(mut self, s: &str) -> anyhow::Result<()> {
        self.encoder.encode_str(self.inner, s)
    }

    fn encode_bytes(mut self, s: &[u8]) -> anyhow::Result<()> {
        self.encoder.encode_bytes(self.inner, s)
    }

    fn encode_none(mut self) -> anyhow::Result<()> {
        self.encoder.encode_none(self.inner)
    }

    fn encode_some(mut self) -> anyhow::Result<<SimpleEncoderAdapter<T> as Encoder>::SomeEncoder<'w>> {
        let (any, closer) = self.encoder.encode_some(self.inner)?;
        Ok(SimpleSomeEncoder {
            encoder: self.encoder,
            some_encoder: Some(any),
            some_closer: Some(closer),
        })
    }

    fn encode_unit_struct(mut self, name: &'static str) -> anyhow::Result<()> {
        self.encoder.encode_unit_struct(self.inner, name)
    }

    fn encode_tuple_struct(
        mut self,
        name: &'static str,
        len: usize,
    ) -> anyhow::Result<<SimpleEncoderAdapter<T> as Encoder>::TupleStructEncoder<'w>> {
        let inner = self.encoder.encode_tuple_struct(self.inner, name, len)?;
        Ok(SimpleTupleStructEncoder {
            encoder: self.encoder,
            inner,
        })
    }

    fn encode_struct(
        mut self,
        name: &'static str,
        fields: &'static [&'static str],
    ) -> anyhow::Result<<SimpleEncoderAdapter<T> as Encoder>::StructEncoder<'w>> {
        let inner = self.encoder.encode_struct(self.inner, name, fields)?;
        Ok(SimpleStructEncoder {
            encoder: self.encoder,
            fields: fields.iter(),
            inner,
        })
    }

    fn encode_unit_variant(
        mut self,
        name: &'static str,
        variants: &'static [&'static str],
        variant_index: usize,
    ) -> anyhow::Result<()> {
        self.encoder
            .encode_unit_variant(self.inner, name, variants, variant_index)
    }

    fn encode_tuple_variant(
        mut self,
        name: &'static str,
        variants: &'static [&'static str],
        variant_index: usize,
        len: usize,
    ) -> anyhow::Result<<SimpleEncoderAdapter<T> as Encoder>::TupleVariantEncoder<'w>> {
        let inner =
            self.encoder
                .encode_tuple_variant(self.inner, name, variants, variant_index, len)?;
        Ok(SimpleTupleVariantEncoder {
            encoder: self.encoder,
            inner,
        })
    }

    fn encode_struct_variant(
        mut self,
        name: &'static str,
        variants: &'static [&'static str],
        variant_index: usize,
        fields: &'static [&'static str],
    ) -> anyhow::Result<<SimpleEncoderAdapter<T> as Encoder>::StructVariantEncoder<'w>> {
        let inner =
            self.encoder
                .encode_struct_variant(self.inner, name, variants, variant_index, fields)?;
        Ok(SimpleStructVariantEncoder {
            encoder: self.encoder,
            inner,
            fields,
        })
    }

    fn encode_seq(
        mut self,
        len: Option<usize>,
    ) -> anyhow::Result<<SimpleEncoderAdapter<T> as Encoder>::SeqEncoder<'w>> {
        let inner = self.encoder.encode_seq(self.inner, len)?;
        Ok(SimpleSeqEncoder {
            encoder: self.encoder,
            inner,
        })
    }

    fn encode_tuple(
        mut self,
        len: usize,
    ) -> anyhow::Result<<SimpleEncoderAdapter<T> as Encoder>::TupleEncoder<'w>> {
        let inner = self.encoder.encode_tuple(self.inner, len)?;
        Ok(SimpleTupleEncoder {
            encoder: self.encoder,
            inner,
        })
    }

    fn encode_map(
        mut self,
        len: Option<usize>,
    ) -> anyhow::Result<<SimpleEncoderAdapter<T> as Encoder>::MapEncoder<'w>> {
        let inner = self.encoder.encode_map(self.inner, len)?;
        Ok(SimpleMapEncoder {
            encoder: self.encoder,
            inner,
        })
    }
}

pub struct SimpleSomeEncoder<'w, T: SimpleEncoder> {
    encoder: &'w mut T,
    some_encoder: Option<T::AnyEncoder>,
    some_closer: Option<T::SomeCloser>,
}

impl<'w, T: SimpleEncoder> SomeEncoder<'w, SimpleEncoderAdapter<T>> for SimpleSomeEncoder<'w, T> {
    fn encode_some(&mut self) -> anyhow::Result<<SimpleEncoderAdapter<T> as Encoder>::AnyEncoder<'_>> {
        Ok(SimpleAnyEncoder {
            encoder: self.encoder,
            inner: self.some_encoder.take().unwrap(),
        })
    }

    fn end(mut self) -> anyhow::Result<()> {
        self.encoder.some_end(self.some_closer.take().unwrap())
    }
}

pub struct SimpleTupleEncoder<'w, T: SimpleEncoder> {
    encoder: &'w mut T,
    inner: T::TupleEncoder,
}

impl<'w, T: SimpleEncoder> TupleEncoder<'w, SimpleEncoderAdapter<T>> for SimpleTupleEncoder<'w, T> {
    fn encode_element(
        &mut self,
    ) -> anyhow::Result<<SimpleEncoderAdapter<T> as Encoder>::AnyEncoder<'_>> {
        let inner = self.encoder.tuple_encode_element(&mut self.inner)?;
        Ok(SimpleAnyEncoder {
            encoder: self.encoder,
            inner,
        })
    }

    fn end(self) -> anyhow::Result<()> {
        self.encoder.tuple_end(self.inner)
    }
}

pub struct SimpleSeqEncoder<'w, T: SimpleEncoder> {
    encoder: &'w mut T,
    inner: T::SeqEncoder,
}

impl<'w, T: SimpleEncoder> SeqEncoder<'w, SimpleEncoderAdapter<T>> for SimpleSeqEncoder<'w, T> {
    fn encode_element(
        &mut self,
    ) -> anyhow::Result<<SimpleEncoderAdapter<T> as Encoder>::AnyEncoder<'_>> {
        let inner = self.encoder.seq_encode_element(&mut self.inner)?;
        Ok(SimpleAnyEncoder {
            encoder: self.encoder,
            inner,
        })
    }

    fn end(self) -> anyhow::Result<()> {
        self.encoder.seq_end(self.inner)
    }
}

pub struct SimpleMapEncoder<'w, T: SimpleEncoder> {
    encoder: &'w mut T,
    inner: T::MapEncoder,
}

impl<'w, T: SimpleEncoder> MapEncoder<'w, SimpleEncoderAdapter<T>> for SimpleMapEncoder<'w, T> {
    fn encode_entry(
        &mut self,
    ) -> anyhow::Result<<SimpleEncoderAdapter<T> as Encoder>::EntryEncoder<'_>> {
        let (key, value) = self.encoder.map_encode_element(&mut self.inner)?;
        Ok(SimpleEntryEncoder {
            encoder: self.encoder,
            key: Some(key),
            value: Some(value),
            closer: None,
        })
    }

    fn end(self) -> anyhow::Result<()> {
        self.encoder.map_end(self.inner)
    }
}

pub struct SimpleEntryEncoder<'w, T: SimpleEncoder> {
    encoder: &'w mut T,
    key: Option<T::AnyEncoder>,
    value: Option<T::ValueEncoder>,
    closer: Option<T::EntryCloser>,
}

impl<'w, T: SimpleEncoder> EntryEncoder<'w, SimpleEncoderAdapter<T>> for SimpleEntryEncoder<'w, T> {
    fn encode_key(&mut self) -> anyhow::Result<<SimpleEncoderAdapter<T> as Encoder>::AnyEncoder<'_>> {
        Ok(SimpleAnyEncoder {
            encoder: self.encoder,
            inner: self.key.take().unwrap(),
        })
    }

    fn encode_value(&mut self) -> anyhow::Result<<SimpleEncoderAdapter<T> as Encoder>::AnyEncoder<'_>> {
        let (any, closer) = self.encoder.entry_encode_value(self.value.take().unwrap())?;
        self.closer = Some(closer);
        Ok(SimpleAnyEncoder {
            encoder: self.encoder,
            inner: any,
        })
    }

    fn end(self) -> anyhow::Result<()> {
        self.encoder.entry_end(self.closer.unwrap())
    }
}

pub struct SimpleTupleStructEncoder<'w, T: SimpleEncoder> {
    encoder: &'w mut T,
    inner: T::TupleStructEncoder,
}

impl<'w, T: SimpleEncoder> TupleStructEncoder<'w, SimpleEncoderAdapter<T>>
for SimpleTupleStructEncoder<'w, T>
{
    fn encode_field(&mut self) -> anyhow::Result<<SimpleEncoderAdapter<T> as Encoder>::AnyEncoder<'_>> {
        let inner = self.encoder.tuple_struct_encode_field(&mut self.inner)?;
        Ok(SimpleAnyEncoder {
            encoder: self.encoder,
            inner,
        })
    }

    fn end(self) -> anyhow::Result<()> {
        self.encoder.tuple_struct_end(self.inner)
    }
}

pub struct SimpleStructEncoder<'w, T: SimpleEncoder> {
    encoder: &'w mut T,
    fields: slice::Iter<'static, &'static str>,
    inner: T::StructEncoder,
}

impl<'w, T: SimpleEncoder> StructEncoder<'w, SimpleEncoderAdapter<T>> for SimpleStructEncoder<'w, T> {
    fn encode_field(&mut self) -> anyhow::Result<<SimpleEncoderAdapter<T> as Encoder>::AnyEncoder<'_>> {
        let inner = self
            .encoder
            .struct_encode_field(&mut self.inner, self.fields.next().unwrap())?;
        Ok(SimpleAnyEncoder {
            encoder: self.encoder,
            inner,
        })
    }

    fn end(self) -> anyhow::Result<()> {
        self.encoder.struct_end(self.inner)
    }
}

pub struct SimpleTupleVariantEncoder<'w, T: SimpleEncoder> {
    encoder: &'w mut T,
    inner: T::TupleVariantEncoder,
}

impl<'w, T: SimpleEncoder> TupleVariantEncoder<'w, SimpleEncoderAdapter<T>>
for SimpleTupleVariantEncoder<'w, T>
{
    fn encode_field(&mut self) -> anyhow::Result<<SimpleEncoderAdapter<T> as Encoder>::AnyEncoder<'_>> {
        let inner = self.encoder.tuple_variant_encode_field(&mut self.inner)?;
        Ok(SimpleAnyEncoder {
            encoder: self.encoder,
            inner,
        })
    }

    fn end(self) -> anyhow::Result<()> {
        self.encoder.tuple_variant_end(self.inner)
    }
}

pub struct SimpleStructVariantEncoder<'w, T: SimpleEncoder> {
    encoder: &'w mut T,
    inner: T::StructVariantEncoder,
    fields: &'static [&'static str],
}

impl<'w, T: SimpleEncoder> StructVariantEncoder<'w, SimpleEncoderAdapter<T>>
for SimpleStructVariantEncoder<'w, T>
{
    fn encode_field(&mut self) -> anyhow::Result<<SimpleEncoderAdapter<T> as Encoder>::AnyEncoder<'_>> {
        let inner = self
            .encoder
            .struct_variant_encode_field(&mut self.inner, self.fields.take_first().unwrap())?;
        Ok(SimpleAnyEncoder {
            encoder: self.encoder,
            inner,
        })
    }

    fn end(self) -> anyhow::Result<()> {
        self.encoder.struct_variant_end(self.inner)
    }
}

impl<T: SimpleEncoder> Encoder for SimpleEncoderAdapter<T> {
    type AnyEncoder<'w> = SimpleAnyEncoder<'w, T> where Self: 'w;
    type SomeEncoder<'w> = SimpleSomeEncoder<'w, T> where Self: 'w;
    type TupleEncoder<'w> = SimpleTupleEncoder<'w, T> where Self: 'w;
    type SeqEncoder<'w> = SimpleSeqEncoder<'w, T> where Self: 'w;
    type MapEncoder<'w> = SimpleMapEncoder<'w, T> where Self: 'w;
    type EntryEncoder<'w> = SimpleEntryEncoder<'w, T> where Self: 'w;
    type TupleStructEncoder<'w> = SimpleTupleStructEncoder<'w, T> where Self: 'w;
    type StructEncoder<'w> = SimpleStructEncoder<'w, T> where Self: 'w;
    type TupleVariantEncoder<'w> = SimpleTupleVariantEncoder<'w, T> where Self: 'w;
    type StructVariantEncoder<'w> = SimpleStructVariantEncoder<'w, T> where Self: 'w;
}
