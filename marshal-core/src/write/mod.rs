use crate::Primitive;

pub mod simple;

pub trait Writer: Sized {
    type AnyWriter<'w>: AnyWriter<'w, Self>
    where
        Self: 'w;
    type SomeWriter<'w>: SomeWriter<'w, Self>
    where
        Self: 'w;
    type TupleWriter<'w>: TupleWriter<'w, Self>
    where
        Self: 'w;
    type SeqWriter<'w>: SeqWriter<'w, Self>
    where
        Self: 'w;
    type MapWriter<'w>: MapWriter<'w, Self>
    where
        Self: 'w;
    type EntryWriter<'w>: EntryWriter<'w, Self>
    where
        Self: 'w;
    type TupleStructWriter<'w>: TupleStructWriter<'w, Self>
    where
        Self: 'w;
    type StructWriter<'w>: StructWriter<'w, Self>
    where
        Self: 'w;
    type TupleVariantWriter<'w>: TupleVariantWriter<'w, Self>
    where
        Self: 'w;
    type StructVariantWriter<'w>: StructVariantWriter<'w, Self>
    where
        Self: 'w;
}

pub trait AnyWriter<'w, W: Writer> {
    fn write_prim(self, prim: Primitive) -> anyhow::Result<()>;
    fn write_str(self, s: &str) -> anyhow::Result<()>;
    fn write_bytes(self, s: &[u8]) -> anyhow::Result<()>;
    fn write_none(self) -> anyhow::Result<()>;
    fn write_some(self) -> anyhow::Result<<W as Writer>::SomeWriter<'w>>;
    fn write_unit_struct(self, name: &'static str) -> anyhow::Result<()>;
    fn write_tuple_struct(
        self,
        name: &'static str,
        len: usize,
    ) -> anyhow::Result<<W as Writer>::TupleStructWriter<'w>>;
    fn write_struct(
        self,
        name: &'static str,
        fields: &'static [&'static str],
    ) -> anyhow::Result<<W as Writer>::StructWriter<'w>>;
    fn write_unit_variant(
        self,
        name: &'static str,
        variants: &'static [&'static str],
        variant_index: u32,
    ) -> anyhow::Result<()>;
    fn write_tuple_variant(
        self,
        name: &'static str,
        variants: &'static [&'static str],
        variant_index: u32,
        len: usize,
    ) -> anyhow::Result<<W as Writer>::TupleVariantWriter<'w>>;
    fn write_struct_variant(
        self,
        name: &'static str,
        variants: &'static [&'static str],
        variant_index: u32,
        len: usize,
    ) -> anyhow::Result<<W as Writer>::StructVariantWriter<'w>>;
    fn write_seq(self, len: Option<usize>) -> anyhow::Result<<W as Writer>::SeqWriter<'w>>;
    fn write_tuple(self, len: usize) -> anyhow::Result<<W as Writer>::TupleWriter<'w>>;
    fn write_map(self, len: Option<usize>) -> anyhow::Result<<W as Writer>::MapWriter<'w>>;
}

pub trait SomeWriter<'w, W: Writer> {
    fn write_some(&mut self) -> anyhow::Result<<W as Writer>::AnyWriter<'_>>;
    fn end(self) -> anyhow::Result<()>;
}

pub trait TupleWriter<'w, W: Writer> {
    fn write_element(&mut self) -> anyhow::Result<<W as Writer>::AnyWriter<'_>>;
    fn end(self) -> anyhow::Result<()>;
}
pub trait SeqWriter<'w, W: Writer> {
    fn write_element(&mut self) -> anyhow::Result<<W as Writer>::AnyWriter<'_>>;
    fn end(self) -> anyhow::Result<()>;
}

pub trait MapWriter<'w, W: Writer> {
    fn write_entry(&mut self) -> anyhow::Result<<W as Writer>::EntryWriter<'_>>;
    fn end(self) -> anyhow::Result<()>;
}

pub trait EntryWriter<'w, W: Writer> {
    fn write_key(&mut self) -> anyhow::Result<<W as Writer>::AnyWriter<'_>>;
    fn write_value(&mut self) -> anyhow::Result<<W as Writer>::AnyWriter<'_>>;
    fn end(self) -> anyhow::Result<()>;
}

pub trait TupleStructWriter<'w, W: Writer> {
    fn write_field(&mut self) -> anyhow::Result<<W as Writer>::AnyWriter<'_>>;
    fn end(self) -> anyhow::Result<()>;
}

pub trait StructWriter<'w, W: Writer> {
    fn write_field(&mut self) -> anyhow::Result<<W as Writer>::AnyWriter<'_>>;
    fn end(self) -> anyhow::Result<()>;
}

pub trait TupleVariantWriter<'w, W: Writer> {
    fn write_field(&mut self) -> anyhow::Result<<W as Writer>::AnyWriter<'_>>;
    fn end(self) -> anyhow::Result<()>;
}

pub trait StructVariantWriter<'w, W: Writer> {
    fn write_field(&mut self, key: &'static str) -> anyhow::Result<<W as Writer>::AnyWriter<'_>>;
    fn end(self) -> anyhow::Result<()>;
}
