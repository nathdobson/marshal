use crate::Primitive;

pub mod simple;

pub trait Encoder: Sized {
    type AnyEncoder<'w>: AnyEncoder<'w, Self>
        where
            Self: 'w;
    type SomeEncoder<'w>: SomeEncoder<'w, Self>
        where
            Self: 'w;
    type TupleEncoder<'w>: TupleEncoder<'w, Self>
        where
            Self: 'w;
    type SeqEncoder<'w>: SeqEncoder<'w, Self>
        where
            Self: 'w;
    type MapEncoder<'w>: MapEncoder<'w, Self>
        where
            Self: 'w;
    type EntryEncoder<'w>: EntryEncoder<'w, Self>
        where
            Self: 'w;
    type TupleStructEncoder<'w>: TupleStructEncoder<'w, Self>
        where
            Self: 'w;
    type StructEncoder<'w>: StructEncoder<'w, Self>
        where
            Self: 'w;
    type TupleVariantEncoder<'w>: TupleVariantEncoder<'w, Self>
        where
            Self: 'w;
    type StructVariantEncoder<'w>: StructVariantEncoder<'w, Self>
        where
            Self: 'w;
}

pub trait AnyEncoder<'w, W: Encoder> {
    fn encode_prim(self, prim: Primitive) -> anyhow::Result<()>;
    fn encode_str(self, s: &str) -> anyhow::Result<()>;
    fn encode_bytes(self, s: &[u8]) -> anyhow::Result<()>;
    fn encode_none(self) -> anyhow::Result<()>;
    fn encode_some(self) -> anyhow::Result<<W as Encoder>::SomeEncoder<'w>>;
    fn encode_unit_struct(self, name: &'static str) -> anyhow::Result<()>;
    fn encode_tuple_struct(
        self,
        name: &'static str,
        len: usize,
    ) -> anyhow::Result<<W as Encoder>::TupleStructEncoder<'w>>;
    fn encode_struct(
        self,
        name: &'static str,
        fields: &'static [&'static str],
    ) -> anyhow::Result<<W as Encoder>::StructEncoder<'w>>;
    fn encode_unit_variant(
        self,
        name: &'static str,
        variants: &'static [&'static str],
        variant_index: u32,
    ) -> anyhow::Result<()>;
    fn encode_tuple_variant(
        self,
        name: &'static str,
        variants: &'static [&'static str],
        variant_index: u32,
        len: usize,
    ) -> anyhow::Result<<W as Encoder>::TupleVariantEncoder<'w>>;
    fn encode_struct_variant(
        self,
        name: &'static str,
        variants: &'static [&'static str],
        variant_index: u32,
        fields: &'static [&'static str],
    ) -> anyhow::Result<<W as Encoder>::StructVariantEncoder<'w>>;
    fn encode_seq(self, len: Option<usize>) -> anyhow::Result<<W as Encoder>::SeqEncoder<'w>>;
    fn encode_tuple(self, len: usize) -> anyhow::Result<<W as Encoder>::TupleEncoder<'w>>;
    fn encode_map(self, len: Option<usize>) -> anyhow::Result<<W as Encoder>::MapEncoder<'w>>;
}

pub trait SomeEncoder<'w, W: Encoder> {
    fn encode_some(&mut self) -> anyhow::Result<<W as Encoder>::AnyEncoder<'_>>;
    fn end(self) -> anyhow::Result<()>;
}

pub trait TupleEncoder<'w, W: Encoder> {
    fn encode_element(&mut self) -> anyhow::Result<<W as Encoder>::AnyEncoder<'_>>;
    fn end(self) -> anyhow::Result<()>;
}

pub trait SeqEncoder<'w, W: Encoder> {
    fn encode_element(&mut self) -> anyhow::Result<<W as Encoder>::AnyEncoder<'_>>;
    fn end(self) -> anyhow::Result<()>;
}

pub trait MapEncoder<'w, W: Encoder> {
    fn encode_entry(&mut self) -> anyhow::Result<<W as Encoder>::EntryEncoder<'_>>;
    fn end(self) -> anyhow::Result<()>;
}

pub trait EntryEncoder<'w, W: Encoder> {
    fn encode_key(&mut self) -> anyhow::Result<<W as Encoder>::AnyEncoder<'_>>;
    fn encode_value(&mut self) -> anyhow::Result<<W as Encoder>::AnyEncoder<'_>>;
    fn end(self) -> anyhow::Result<()>;
}

pub trait TupleStructEncoder<'w, W: Encoder> {
    fn encode_field(&mut self) -> anyhow::Result<<W as Encoder>::AnyEncoder<'_>>;
    fn end(self) -> anyhow::Result<()>;
}

pub trait StructEncoder<'w, W: Encoder> {
    fn encode_field(&mut self) -> anyhow::Result<<W as Encoder>::AnyEncoder<'_>>;
    fn end(self) -> anyhow::Result<()>;
}

pub trait TupleVariantEncoder<'w, W: Encoder> {
    fn encode_field(&mut self) -> anyhow::Result<<W as Encoder>::AnyEncoder<'_>>;
    fn end(self) -> anyhow::Result<()>;
}

pub trait StructVariantEncoder<'w, W: Encoder> {
    fn encode_field(&mut self) -> anyhow::Result<<W as Encoder>::AnyEncoder<'_>>;
    fn end(self) -> anyhow::Result<()>;
}
