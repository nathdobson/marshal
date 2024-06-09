#![feature(core_intrinsics)]
#![feature(ptr_metadata)]
#![feature(adt_const_params)]
#![feature(const_heap)]
#![feature(const_mut_refs)]
#![allow(incomplete_features)]
#![allow(internal_features)]

#[cfg(test)]
mod test;

use marshal::context::Context;
use marshal::de::{Deserialize, SchemaError};
use marshal::ser::Serialize;
use marshal_core::decode::{
    AnyDecoder, DecodeHint, Decoder, DecoderView, EntryDecoder, MapDecoder,
};
use marshal_core::encode::{AnyEncoder, Encoder, StructEncoder};
use std::intrinsics::const_allocate;
use std::mem::{align_of, size_of};

#[derive(Default, Eq, Ord, PartialEq, PartialOrd, Debug, Hash, Copy, Clone)]
pub struct StructNil<const STRUCT: &'static str>;

#[derive(Default, Eq, Ord, PartialEq, PartialOrd, Debug, Hash, Copy, Clone)]
pub struct StructCons<const FIELD: &'static str, H, T> {
    pub head: H,
    pub tail: T,
}

impl<const STRUCT: &'static str> StructNil<STRUCT> {
    pub fn new() -> Self {
        StructNil
    }
}

impl<const FIELD: &'static str, H, T> StructCons<FIELD, H, T> {
    pub fn new(head: H, tail: T) -> Self {
        StructCons { head, tail }
    }
}

#[rustfmt::skip]
pub trait StructList {
    const STRUCT: &'static str;
    const LEN: usize;
    const FIELDS: &'static [&'static str];
}

impl<const STRUCT: &'static str> StructList for StructNil<STRUCT> {
    const STRUCT: &'static str = STRUCT;
    const LEN: usize = 0;
    const FIELDS: &'static [&'static str] = &[];
}

impl<const FIELD: &'static str, H, T: StructList> StructList for StructCons<FIELD, H, T> {
    const STRUCT: &'static str = T::STRUCT;
    const LEN: usize = T::LEN + 1;
    const FIELDS: &'static [&'static str] = unsafe {
        let len = T::FIELDS.len() + 1;
        let output = const_allocate(len * size_of::<&'static str>(), align_of::<&'static str>());
        let output: *mut [&'static str] = std::ptr::from_raw_parts_mut(output as *mut (), len);
        let output: &mut [&'static str] = &mut *output;
        let mut index = 0;
        while index < T::FIELDS.len() {
            output[index + 1] = T::FIELDS[index];
            index += 1;
        }
        output[0] = FIELD;
        &*output
    };
}

trait SerializeStructList<W: Encoder>: StructList {
    fn serialize_struct_list(
        &self,
        _: W::StructEncoder<'_>,
        ctx: &mut Context,
    ) -> anyhow::Result<()>;
}

impl<const STRUCT: &'static str, W: Encoder> SerializeStructList<W> for StructNil<STRUCT> {
    fn serialize_struct_list(
        &self,
        e: W::StructEncoder<'_>,
        _ctx: &mut Context,
    ) -> anyhow::Result<()> {
        e.end()
    }
}

impl<const FIELD: &'static str, W: Encoder, H: Serialize<W>, T: SerializeStructList<W>>
    SerializeStructList<W> for StructCons<FIELD, H, T>
{
    fn serialize_struct_list(
        &self,
        mut e: W::StructEncoder<'_>,
        ctx: &mut Context,
    ) -> anyhow::Result<()> {
        self.head.serialize(e.encode_field()?, ctx)?;
        self.tail.serialize_struct_list(e, ctx)
    }
}

impl<const STRUCT: &'static str, W: Encoder> Serialize<W> for StructNil<STRUCT>
where
    Self: SerializeStructList<W>,
{
    fn serialize(&self, w: W::AnyEncoder<'_>, ctx: &mut Context) -> anyhow::Result<()> {
        let w = w.encode_struct(Self::STRUCT, Self::FIELDS)?;
        self.serialize_struct_list(w, ctx)?;
        Ok(())
    }
}

impl<const FIELD: &'static str, W: Encoder, H, T> Serialize<W> for StructCons<FIELD, H, T>
where
    Self: SerializeStructList<W>,
{
    fn serialize(&self, w: W::AnyEncoder<'_>, ctx: &mut Context) -> anyhow::Result<()> {
        let w = w.encode_struct(Self::STRUCT, Self::FIELDS)?;
        self.serialize_struct_list(w, ctx)?;
        Ok(())
    }
}

trait DeserializeStructList<'de, D: Decoder<'de>>: StructList + Sized {
    type Builder: Default;
    fn decode_field(
        builder: &mut Self::Builder,
        field: &str,
        value: D::AnyDecoder<'_>,
        ctx: &mut Context,
    ) -> anyhow::Result<()>;
    fn build(builder: Self::Builder) -> anyhow::Result<Self>;
    fn deserialize_struct_list(d: D::AnyDecoder<'_>, ctx: &mut Context) -> anyhow::Result<Self> {
        let mut builder = Self::Builder::default();
        match d.decode(DecodeHint::Struct {
            name: Self::STRUCT,
            fields: Self::FIELDS,
        })? {
            DecoderView::Map(mut d) => {
                while let Some(mut d) = d.decode_next()? {
                    let field = match d.decode_key()?.decode(DecodeHint::Identifier)? {
                        DecoderView::String(x) => x,
                        _ => todo!(),
                    };
                    Self::decode_field(&mut builder, &*field, d.decode_value()?, ctx)?;
                    d.decode_end()?;
                }
            }
            _ => todo!(),
        }
        Self::build(builder)
    }
}

impl<'de, const STRUCT: &'static str, D: Decoder<'de>> DeserializeStructList<'de, D>
    for StructNil<STRUCT>
{
    type Builder = StructNil<STRUCT>;

    fn decode_field(
        _: &mut Self::Builder,
        _: &str,
        value: D::AnyDecoder<'_>,
        _: &mut Context,
    ) -> anyhow::Result<()> {
        value.ignore()
    }

    fn build(builder: Self::Builder) -> anyhow::Result<Self> {
        Ok(builder)
    }
}

impl<
        'de,
        const FIELD: &'static str,
        D: Decoder<'de>,
        H: Deserialize<'de, D>,
        T: DeserializeStructList<'de, D>,
    > DeserializeStructList<'de, D> for StructCons<FIELD, H, T>
{
    type Builder = StructCons<FIELD, Option<H>, T::Builder>;

    fn decode_field(
        builder: &mut Self::Builder,
        field: &str,
        value: D::AnyDecoder<'_>,
        ctx: &mut Context,
    ) -> anyhow::Result<()> {
        if field == FIELD {
            builder.head = Some(H::deserialize(value, ctx)?);
            Ok(())
        } else {
            T::decode_field(&mut builder.tail, field, value, ctx)
        }
    }

    fn build(builder: Self::Builder) -> anyhow::Result<Self> {
        Ok(Self::new(
            builder
                .head
                .ok_or(SchemaError::MissingField { field_name: FIELD })?,
            T::build(builder.tail)?,
        ))
    }
}

impl<'de, const STRUCT: &'static str, D: Decoder<'de>> Deserialize<'de, D> for StructNil<STRUCT>
where
    Self: DeserializeStructList<'de, D>,
{
    fn deserialize<'p>(d: D::AnyDecoder<'p>, ctx: &mut Context) -> anyhow::Result<Self> {
        Self::deserialize_struct_list(d, ctx)
    }
}

impl<'de, const FIELD: &'static str, D: Decoder<'de>, H, T> Deserialize<'de, D>
    for StructCons<FIELD, H, T>
where
    Self: DeserializeStructList<'de, D>,
{
    fn deserialize<'p>(d: D::AnyDecoder<'p>, ctx: &mut Context) -> anyhow::Result<Self> {
        Self::deserialize_struct_list(d, ctx)
    }
}