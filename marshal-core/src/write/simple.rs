use std::marker::PhantomData;

use crate::Primitive;
use crate::write::{AnyWriter, EntryWriter, MapWriter, SeqWriter, SomeWriter, StructVariantWriter, StructWriter, TupleStructWriter, TupleVariantWriter, TupleWriter, Writer};

pub trait SimpleWriter {
    type AnyWriter;
    type SomeCloser;
    type TupleWriter;
    type SeqWriter;
    type MapWriter;
    type ValueWriter;
    type EntryCloser;
    type TupleStructWriter;
    type StructWriter;
    type TupleVariantWriter;
    type StructVariantWriter;
    fn write_prim(&mut self, any: Self::AnyWriter, prim: Primitive) -> anyhow::Result<()>;
    fn write_str(&mut self, any: Self::AnyWriter, s: &str) -> anyhow::Result<()>;
    fn write_bytes(&mut self, any: Self::AnyWriter, s: &[u8]) -> anyhow::Result<()>;
    fn write_none(&mut self, any: Self::AnyWriter) -> anyhow::Result<()>;
    fn write_some(
        &mut self,
        any: Self::AnyWriter,
    ) -> anyhow::Result<(Self::AnyWriter, Self::SomeCloser)>;
    fn write_unit_struct(&mut self, any: Self::AnyWriter, name: &'static str)
        -> anyhow::Result<()>;
    fn write_tuple_struct(
        &mut self,
        any: Self::AnyWriter,
        name: &'static str,
        len: usize,
    ) -> anyhow::Result<Self::TupleStructWriter>;
    fn write_struct(
        &mut self,
        any: Self::AnyWriter,
        name: &'static str,
        len: usize,
    ) -> anyhow::Result<Self::StructWriter>;
    fn write_unit_variant(
        &mut self,
        any: Self::AnyWriter,
        name: &'static str,
        variant_index: u32,
        variant: &'static str,
    ) -> anyhow::Result<()>;
    fn write_tuple_variant(
        &mut self,
        any: Self::AnyWriter,
        name: &'static str,
        variant_index: u32,
        variant: &'static str,
        len: usize,
    ) -> anyhow::Result<Self::TupleVariantWriter>;
    fn write_struct_variant(
        &mut self,
        any: Self::AnyWriter,
        name: &'static str,
        variant_index: u32,
        variant: &'static str,
        len: usize,
    ) -> anyhow::Result<Self::StructVariantWriter>;
    fn write_seq(
        &mut self,
        any: Self::AnyWriter,
        len: Option<usize>,
    ) -> anyhow::Result<Self::SeqWriter>;
    fn write_tuple(
        &mut self,
        any: Self::AnyWriter,
        len: usize,
    ) -> anyhow::Result<Self::TupleWriter>;
    fn write_map(
        &mut self,
        any: Self::AnyWriter,
        len: Option<usize>,
    ) -> anyhow::Result<Self::MapWriter>;

    fn some_end(&mut self, some: Self::SomeCloser) -> anyhow::Result<()>;

    fn tuple_write_element(
        &mut self,
        tuple: &mut Self::TupleWriter,
    ) -> anyhow::Result<Self::AnyWriter>;
    fn tuple_end(&mut self, tuple: Self::TupleWriter) -> anyhow::Result<()>;

    fn seq_write_element(&mut self, seq: &mut Self::SeqWriter)
        -> anyhow::Result<Self::AnyWriter>;
    fn seq_end(&mut self, tuple: Self::SeqWriter) -> anyhow::Result<()>;

    fn map_write_element(
        &mut self,
        map: &mut Self::MapWriter,
    ) -> anyhow::Result<(Self::AnyWriter, Self::ValueWriter)>;
    fn map_end(&mut self, map: Self::MapWriter) -> anyhow::Result<()>;

    fn entry_write_value(
        &mut self,
        value: Self::ValueWriter,
    ) -> anyhow::Result<(Self::AnyWriter, Self::EntryCloser)>;
    fn entry_end(&mut self, closer: Self::EntryCloser) -> anyhow::Result<()>;

    fn tuple_struct_write_field(
        &mut self,
        map: &mut Self::TupleStructWriter,
    ) -> anyhow::Result<Self::AnyWriter>;
    fn tuple_struct_end(&mut self, map: Self::TupleStructWriter) -> anyhow::Result<()>;

    fn struct_write_field(
        &mut self,
        map: &mut Self::StructWriter,
        key: &'static str,
    ) -> anyhow::Result<Self::AnyWriter>;
    fn struct_end(&mut self, map: Self::StructWriter) -> anyhow::Result<()>;

    fn tuple_variant_write_field(
        &mut self,
        map: &mut Self::TupleVariantWriter,
    ) -> anyhow::Result<Self::AnyWriter>;
    fn tuple_variant_end(&mut self, map: Self::TupleVariantWriter) -> anyhow::Result<()>;

    fn struct_variant_write_field(
        &mut self,
        map: &mut Self::StructVariantWriter,
        key: &'static str,
    ) -> anyhow::Result<Self::AnyWriter>;
    fn struct_variant_end(&mut self, map: Self::StructVariantWriter) -> anyhow::Result<()>;
}

pub struct SimpleWriterAdapter<T>(PhantomData<T>);

pub struct SimpleAnyWriter<'w, T: SimpleWriter> {
    writer: &'w mut T,
    inner: T::AnyWriter,
}

impl<'w, T: SimpleWriter> SimpleAnyWriter<'w, T> {
    pub fn new(writer: &'w mut T, inner: T::AnyWriter) -> Self {
        SimpleAnyWriter { writer, inner }
    }
}

impl<'w, T: SimpleWriter> AnyWriter<'w, SimpleWriterAdapter<T>> for SimpleAnyWriter<'w, T> {
    fn write_prim(mut self, prim: Primitive) -> anyhow::Result<()> {
        self.writer.write_prim(self.inner, prim)
    }

    fn write_str(mut self, s: &str) -> anyhow::Result<()> {
        self.writer.write_str(self.inner, s)
    }

    fn write_bytes(mut self, s: &[u8]) -> anyhow::Result<()> {
        self.writer.write_bytes(self.inner, s)
    }

    fn write_none(mut self) -> anyhow::Result<()> {
        self.writer.write_none(self.inner)
    }

    fn write_some(mut self) -> anyhow::Result<<SimpleWriterAdapter<T> as Writer>::SomeWriter<'w>> {
        let (any, closer) = self.writer.write_some(self.inner)?;
        Ok(SimpleSomeWriter {
            writer: self.writer,
            some_writer: Some(any),
            some_closer: Some(closer),
        })
    }

    fn write_unit_struct(mut self, name: &'static str) -> anyhow::Result<()> {
        self.writer.write_unit_struct(self.inner, name)
    }

    fn write_tuple_struct(
        mut self,
        name: &'static str,
        len: usize,
    ) -> anyhow::Result<<SimpleWriterAdapter<T> as Writer>::TupleStructWriter<'w>> {
        let inner = self.writer.write_tuple_struct(self.inner, name, len)?;
        Ok(SimpleTupleStructWriter {
            writer: self.writer,
            inner,
        })
    }

    fn write_struct(
        mut self,
        name: &'static str,
        len: usize,
    ) -> anyhow::Result<<SimpleWriterAdapter<T> as Writer>::StructWriter<'w>> {
        let inner = self.writer.write_struct(self.inner, name, len)?;
        Ok(SimpleStructWriter {
            writer: self.writer,
            inner,
        })
    }

    fn write_unit_variant(
        mut self,
        name: &'static str,
        variant_index: u32,
        variant: &'static str,
    ) -> anyhow::Result<()> {
        self.writer
            .write_unit_variant(self.inner, name, variant_index, variant)
    }

    fn write_tuple_variant(
        mut self,
        name: &'static str,
        variant_index: u32,
        variant: &'static str,
        len: usize,
    ) -> anyhow::Result<<SimpleWriterAdapter<T> as Writer>::TupleVariantWriter<'w>> {
        let inner =
            self.writer
                .write_tuple_variant(self.inner, name, variant_index, variant, len)?;
        Ok(SimpleTupleVariantWriter {
            writer: self.writer,
            inner,
        })
    }

    fn write_struct_variant(
        mut self,
        name: &'static str,
        variant_index: u32,
        variant: &'static str,
        len: usize,
    ) -> anyhow::Result<<SimpleWriterAdapter<T> as Writer>::StructVariantWriter<'w>> {
        let inner =
            self.writer
                .write_struct_variant(self.inner, name, variant_index, variant, len)?;
        Ok(SimpleStructVariantWriter {
            writer: self.writer,
            inner,
        })
    }

    fn write_seq(
        mut self,
        len: Option<usize>,
    ) -> anyhow::Result<<SimpleWriterAdapter<T> as Writer>::SeqWriter<'w>> {
        let inner = self.writer.write_seq(self.inner, len)?;
        Ok(SimpleSeqWriter {
            writer: self.writer,
            inner,
        })
    }

    fn write_tuple(
        mut self,
        len: usize,
    ) -> anyhow::Result<<SimpleWriterAdapter<T> as Writer>::TupleWriter<'w>> {
        let inner = self.writer.write_tuple(self.inner, len)?;
        Ok(SimpleTupleWriter {
            writer: self.writer,
            inner,
        })
    }

    fn write_map(
        mut self,
        len: Option<usize>,
    ) -> anyhow::Result<<SimpleWriterAdapter<T> as Writer>::MapWriter<'w>> {
        let inner = self.writer.write_map(self.inner, len)?;
        Ok(SimpleMapWriter {
            writer: self.writer,
            inner,
        })
    }
}

pub struct SimpleSomeWriter<'w, T: SimpleWriter> {
    writer: &'w mut T,
    some_writer: Option<T::AnyWriter>,
    some_closer: Option<T::SomeCloser>,
}

impl<'w, T: SimpleWriter> SomeWriter<'w, SimpleWriterAdapter<T>> for SimpleSomeWriter<'w, T> {
    fn write_some(&mut self) -> anyhow::Result<<SimpleWriterAdapter<T> as Writer>::AnyWriter<'_>> {
        Ok(SimpleAnyWriter {
            writer: self.writer,
            inner: self.some_writer.take().unwrap(),
        })
    }

    fn end(mut self) -> anyhow::Result<()> {
        self.writer.some_end(self.some_closer.take().unwrap())
    }
}

pub struct SimpleTupleWriter<'w, T: SimpleWriter> {
    writer: &'w mut T,
    inner: T::TupleWriter,
}

impl<'w, T: SimpleWriter> TupleWriter<'w, SimpleWriterAdapter<T>> for SimpleTupleWriter<'w, T> {
    fn write_element(
        &mut self,
    ) -> anyhow::Result<<SimpleWriterAdapter<T> as Writer>::AnyWriter<'_>> {
        let inner = self.writer.tuple_write_element(&mut self.inner)?;
        Ok(SimpleAnyWriter {
            writer: self.writer,
            inner,
        })
    }

    fn end(self) -> anyhow::Result<()> {
        self.writer.tuple_end(self.inner)
    }
}

pub struct SimpleSeqWriter<'w, T: SimpleWriter> {
    writer: &'w mut T,
    inner: T::SeqWriter,
}

impl<'w, T: SimpleWriter> SeqWriter<'w, SimpleWriterAdapter<T>> for SimpleSeqWriter<'w, T> {
    fn write_element(
        &mut self,
    ) -> anyhow::Result<<SimpleWriterAdapter<T> as Writer>::AnyWriter<'_>> {
        let inner = self.writer.seq_write_element(&mut self.inner)?;
        Ok(SimpleAnyWriter {
            writer: self.writer,
            inner,
        })
    }

    fn end(self) -> anyhow::Result<()> {
        self.writer.seq_end(self.inner)
    }
}

pub struct SimpleMapWriter<'w, T: SimpleWriter> {
    writer: &'w mut T,
    inner: T::MapWriter,
}

impl<'w, T: SimpleWriter> MapWriter<'w, SimpleWriterAdapter<T>> for SimpleMapWriter<'w, T> {
    fn write_entry(
        &mut self,
    ) -> anyhow::Result<<SimpleWriterAdapter<T> as Writer>::EntryWriter<'_>> {
        let (key, value) = self.writer.map_write_element(&mut self.inner)?;
        Ok(SimpleEntryWriter {
            writer: self.writer,
            key: Some(key),
            value: Some(value),
            closer: None,
        })
    }

    fn end(self) -> anyhow::Result<()> {
        self.writer.map_end(self.inner)
    }
}

pub struct SimpleEntryWriter<'w, T: SimpleWriter> {
    writer: &'w mut T,
    key: Option<T::AnyWriter>,
    value: Option<T::ValueWriter>,
    closer: Option<T::EntryCloser>,
}

impl<'w, T: SimpleWriter> EntryWriter<'w, SimpleWriterAdapter<T>> for SimpleEntryWriter<'w, T> {
    fn write_key(&mut self) -> anyhow::Result<<SimpleWriterAdapter<T> as Writer>::AnyWriter<'_>> {
        Ok(SimpleAnyWriter {
            writer: self.writer,
            inner: self.key.take().unwrap(),
        })
    }

    fn write_value(&mut self) -> anyhow::Result<<SimpleWriterAdapter<T> as Writer>::AnyWriter<'_>> {
        let (any, closer) = self.writer.entry_write_value(self.value.take().unwrap())?;
        self.closer = Some(closer);
        Ok(SimpleAnyWriter {
            writer: self.writer,
            inner: any,
        })
    }

    fn end(self) -> anyhow::Result<()> {
        self.writer.entry_end(self.closer.unwrap())
    }
}

pub struct SimpleTupleStructWriter<'w, T: SimpleWriter> {
    writer: &'w mut T,
    inner: T::TupleStructWriter,
}

impl<'w, T: SimpleWriter> TupleStructWriter<'w, SimpleWriterAdapter<T>>
    for SimpleTupleStructWriter<'w, T>
{
    fn write_field(&mut self) -> anyhow::Result<<SimpleWriterAdapter<T> as Writer>::AnyWriter<'_>> {
        let inner = self.writer.tuple_struct_write_field(&mut self.inner)?;
        Ok(SimpleAnyWriter {
            writer: self.writer,
            inner,
        })
    }

    fn end(self) -> anyhow::Result<()> {
        self.writer.tuple_struct_end(self.inner)
    }
}

pub struct SimpleStructWriter<'w, T: SimpleWriter> {
    writer: &'w mut T,
    inner: T::StructWriter,
}

impl<'w, T: SimpleWriter> StructWriter<'w, SimpleWriterAdapter<T>> for SimpleStructWriter<'w, T> {
    fn write_field(
        &mut self,
        key: &'static str,
    ) -> anyhow::Result<<SimpleWriterAdapter<T> as Writer>::AnyWriter<'_>> {
        let inner = self.writer.struct_write_field(&mut self.inner, key)?;
        Ok(SimpleAnyWriter {
            writer: self.writer,
            inner,
        })
    }

    fn end(self) -> anyhow::Result<()> {
        self.writer.struct_end(self.inner)
    }
}

pub struct SimpleTupleVariantWriter<'w, T: SimpleWriter> {
    writer: &'w mut T,
    inner: T::TupleVariantWriter,
}

impl<'w, T: SimpleWriter> TupleVariantWriter<'w, SimpleWriterAdapter<T>>
    for SimpleTupleVariantWriter<'w, T>
{
    fn write_field(&mut self) -> anyhow::Result<<SimpleWriterAdapter<T> as Writer>::AnyWriter<'_>> {
        let inner = self.writer.tuple_variant_write_field(&mut self.inner)?;
        Ok(SimpleAnyWriter {
            writer: self.writer,
            inner,
        })
    }

    fn end(self) -> anyhow::Result<()> {
        self.writer.tuple_variant_end(self.inner)
    }
}

pub struct SimpleStructVariantWriter<'w, T: SimpleWriter> {
    writer: &'w mut T,
    inner: T::StructVariantWriter,
}

impl<'w, T: SimpleWriter> StructVariantWriter<'w, SimpleWriterAdapter<T>>
    for SimpleStructVariantWriter<'w, T>
{
    fn write_field(
        &mut self,
        key: &'static str,
    ) -> anyhow::Result<<SimpleWriterAdapter<T> as Writer>::AnyWriter<'_>> {
        let inner = self
            .writer
            .struct_variant_write_field(&mut self.inner, key)?;
        Ok(SimpleAnyWriter {
            writer: self.writer,
            inner,
        })
    }

    fn end(self) -> anyhow::Result<()> {
        self.writer.struct_variant_end(self.inner)
    }
}

impl<T: SimpleWriter> Writer for SimpleWriterAdapter<T> {
    type AnyWriter<'w> =SimpleAnyWriter<'w,T> where Self: 'w;
    type SomeWriter<'w> =SimpleSomeWriter<'w,T>  where Self: 'w;
    type TupleWriter<'w> =SimpleTupleWriter<'w,T>  where Self: 'w;
    type SeqWriter<'w> =SimpleSeqWriter<'w,T>  where Self: 'w;
    type MapWriter<'w> =SimpleMapWriter<'w,T>  where Self: 'w;
    type EntryWriter<'w> =SimpleEntryWriter<'w,T>  where Self: 'w;
    type TupleStructWriter<'w>  =SimpleTupleStructWriter<'w,T> where Self: 'w;
    type StructWriter<'w>  =SimpleStructWriter<'w,T> where Self: 'w;
    type TupleVariantWriter<'w> =SimpleTupleVariantWriter<'w,T>  where Self: 'w;
    type StructVariantWriter<'w>  =SimpleStructVariantWriter<'w,T> where Self: 'w;
}
