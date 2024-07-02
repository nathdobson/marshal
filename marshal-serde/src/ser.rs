use anyhow::{anyhow, Error};
use marshal::context::Context;
use marshal::encode::{AnyEncoder, AnySpecEncoder, Encoder, SpecEncoder};
use marshal::Primitive;
use serde::{Serialize, Serializer};
use std::fmt::{Debug, Display, Formatter};
use crate::{MarshalError, SerdeWrapper};

struct MarshalSerializer<'w, 'en, E: Encoder>(marshal::encode::AnyEncoder<'w, 'en, E>);

struct MarshalSerializeSeq<'w, 'en, E: Encoder>(
    marshal::encode::SeqEncoder<'w, E::SpecEncoder<'en>>,
);
struct MarshalSerializeTuple<'w, 'en, E: Encoder>(
    marshal::encode::TupleEncoder<'w, E::SpecEncoder<'en>>,
);

struct MarshalSerializeTupleStruct<'w, 'en, E: Encoder>(
    marshal::encode::TupleStructEncoder<'w, E::SpecEncoder<'en>>,
);

struct MarshalSerializeTupleVariant<'w, 'en, E: Encoder> {
    encoder: &'w mut E::SpecEncoder<'en>,
    fields: <E::SpecEncoder<'en> as SpecEncoder>::TupleEncoder,
    entry: <E::SpecEncoder<'en> as SpecEncoder>::EntryCloser,
    map: <E::SpecEncoder<'en> as SpecEncoder>::MapEncoder,
}

struct MarshalSerializeMap<'w, 'en, E: Encoder> {
    encoder: &'w mut E::SpecEncoder<'en>,
    entry: Option<<E::SpecEncoder<'en> as SpecEncoder>::ValueEncoder>,
    map: <E::SpecEncoder<'en> as SpecEncoder>::MapEncoder,
}

struct MarshalSerializeStruct<'w, 'en, E: Encoder>(
    marshal::encode::MapEncoder<'w, E::SpecEncoder<'en>>,
);

struct MarshalSerializeStructVariant<'w, 'en, E: Encoder> {
    encoder: &'w mut E::SpecEncoder<'en>,
    fields: <E::SpecEncoder<'en> as SpecEncoder>::MapEncoder,
    entry: <E::SpecEncoder<'en> as SpecEncoder>::EntryCloser,
    map: <E::SpecEncoder<'en> as SpecEncoder>::MapEncoder,
}


impl<'w, 'en, E: Encoder> Serializer for MarshalSerializer<'w, 'en, E> {
    type Ok = ();
    type Error = MarshalError;
    type SerializeSeq = MarshalSerializeSeq<'w, 'en, E>;
    type SerializeTuple = MarshalSerializeTuple<'w, 'en, E>;
    type SerializeTupleStruct = MarshalSerializeTupleStruct<'w, 'en, E>;
    type SerializeTupleVariant = MarshalSerializeTupleVariant<'w, 'en, E>;
    type SerializeMap = MarshalSerializeMap<'w, 'en, E>;
    type SerializeStruct = MarshalSerializeStruct<'w, 'en, E>;
    type SerializeStructVariant = MarshalSerializeStructVariant<'w, 'en, E>;

    fn serialize_bool(self, v: bool) -> Result<Self::Ok, Self::Error> {
        Ok(self.0.encode_prim(Primitive::Bool(v))?)
    }

    fn serialize_i8(self, v: i8) -> Result<Self::Ok, Self::Error> {
        Ok(self.0.encode_prim(Primitive::I8(v))?)
    }

    fn serialize_i16(self, v: i16) -> Result<Self::Ok, Self::Error> {
        Ok(self.0.encode_prim(Primitive::I16(v))?)
    }

    fn serialize_i32(self, v: i32) -> Result<Self::Ok, Self::Error> {
        Ok(self.0.encode_prim(Primitive::I32(v))?)
    }

    fn serialize_i64(self, v: i64) -> Result<Self::Ok, Self::Error> {
        Ok(self.0.encode_prim(Primitive::I64(v))?)
    }

    fn serialize_i128(self, v: i128) -> Result<Self::Ok, Self::Error> {
        Ok(self.0.encode_prim(Primitive::I128(v))?)
    }

    fn serialize_u8(self, v: u8) -> Result<Self::Ok, Self::Error> {
        Ok(self.0.encode_prim(Primitive::U8(v))?)
    }

    fn serialize_u16(self, v: u16) -> Result<Self::Ok, Self::Error> {
        Ok(self.0.encode_prim(Primitive::U16(v))?)
    }

    fn serialize_u32(self, v: u32) -> Result<Self::Ok, Self::Error> {
        Ok(self.0.encode_prim(Primitive::U32(v))?)
    }

    fn serialize_u64(self, v: u64) -> Result<Self::Ok, Self::Error> {
        Ok(self.0.encode_prim(Primitive::U64(v))?)
    }

    fn serialize_u128(self, v: u128) -> Result<Self::Ok, Self::Error> {
        Ok(self.0.encode_prim(Primitive::U128(v))?)
    }

    fn serialize_f32(self, v: f32) -> Result<Self::Ok, Self::Error> {
        Ok(self.0.encode_prim(Primitive::F32(v))?)
    }

    fn serialize_f64(self, v: f64) -> Result<Self::Ok, Self::Error> {
        Ok(self.0.encode_prim(Primitive::F64(v))?)
    }

    fn serialize_char(self, v: char) -> Result<Self::Ok, Self::Error> {
        Ok(self.0.encode_prim(Primitive::Char(v))?)
    }

    fn serialize_str(self, v: &str) -> Result<Self::Ok, Self::Error> {
        Ok(self.0.encode_str(v)?)
    }

    fn serialize_bytes(self, v: &[u8]) -> Result<Self::Ok, Self::Error> {
        Ok(self.0.encode_bytes(v)?)
    }

    fn serialize_none(self) -> Result<Self::Ok, Self::Error> {
        Ok(self.0.encode_none()?)
    }

    fn serialize_some<T>(self, value: &T) -> Result<Self::Ok, Self::Error>
    where
        T: ?Sized + Serialize,
    {
        let mut e = self.0.encode_some()?;
        value.serialize(MarshalSerializer::<E>(e.encode_some()?))?;
        e.end()?;
        Ok(())
    }

    fn serialize_unit(self) -> Result<Self::Ok, Self::Error> {
        Ok(self.0.encode_prim(Primitive::Unit)?)
    }

    fn serialize_unit_struct(self, name: &'static str) -> Result<Self::Ok, Self::Error> {
        Ok(self.0.encode_unit_struct(name)?)
    }

    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
    ) -> Result<Self::Ok, Self::Error> {
        let mut e = self.0.encode_map(Some(1))?;
        {
            let mut e = e.encode_entry()?;
            e.encode_key()?.encode_str(variant)?;
            e.encode_value()?.encode_prim(Primitive::Unit)?;
            e.end()?;
        }
        e.end()?;
        Ok(())
    }

    fn serialize_newtype_struct<T>(
        self,
        name: &'static str,
        value: &T,
    ) -> Result<Self::Ok, Self::Error>
    where
        T: ?Sized + Serialize,
    {
        let mut e = self.0.encode_tuple_struct(name, 1)?;
        value.serialize(MarshalSerializer::<E>(e.encode_field()?))?;
        e.end()?;
        Ok(())
    }

    fn serialize_newtype_variant<T>(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        value: &T,
    ) -> Result<Self::Ok, Self::Error>
    where
        T: ?Sized + Serialize,
    {
        let mut e = self.0.encode_map(Some(1))?;
        {
            let mut e = e.encode_entry()?;
            e.encode_key()?.encode_str(variant)?;
            value.serialize(MarshalSerializer::<E>(e.encode_value()?))?;
            e.end()?;
        }
        Ok(())
    }

    fn serialize_seq(self, len: Option<usize>) -> Result<Self::SerializeSeq, Self::Error> {
        let e = self.0.encode_seq(len)?;
        Ok(MarshalSerializeSeq(e))
    }

    fn serialize_tuple(self, len: usize) -> Result<Self::SerializeTuple, Self::Error> {
        Ok(MarshalSerializeTuple(self.0.encode_tuple(len)?))
    }

    fn serialize_tuple_struct(
        self,
        name: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleStruct, Self::Error> {
        Ok(MarshalSerializeTupleStruct(
            self.0.encode_tuple_struct(name, len)?,
        ))
    }

    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleVariant, Self::Error> {
        let (encoder, any) = self.0.into_raw();
        let mut map = encoder.encode_map(any, Some(1))?;
        let (key, value) = encoder.map_encode_element(&mut map)?;
        encoder.encode_str(key, variant)?;
        let (value, entry) = encoder.entry_encode_value(value)?;
        let fields = encoder.encode_tuple(value, len)?;
        Ok(MarshalSerializeTupleVariant {
            encoder,
            fields,
            entry,
            map,
        })
    }

    fn serialize_map(self, len: Option<usize>) -> Result<Self::SerializeMap, Self::Error> {
        let (encoder, any) = self.0.into_raw();
        let map = encoder.encode_map(any, len)?;
        Ok(MarshalSerializeMap {
            encoder,
            entry: None,
            map,
        })
    }

    fn serialize_struct(
        self,
        _name: &'static str,
        len: usize,
    ) -> Result<Self::SerializeStruct, Self::Error> {
        Ok(MarshalSerializeStruct(self.0.encode_map(Some(len))?))
    }

    fn serialize_struct_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        len: usize,
    ) -> Result<Self::SerializeStructVariant, Self::Error> {
        let (encoder, any) = self.0.into_raw();
        let mut map = encoder.encode_map(any, Some(1))?;
        let (key, value) = encoder.map_encode_element(&mut map)?;
        encoder.encode_str(key, variant)?;
        let (value, entry) = encoder.entry_encode_value(value)?;
        let fields = encoder.encode_map(value, Some(len))?;
        Ok(MarshalSerializeStructVariant {
            encoder,
            fields,
            entry,
            map,
        })
    }

    fn is_human_readable(&self) -> bool {
        self.0.is_human_readable()
    }
}

impl<'w, 'en, E: Encoder> serde::ser::SerializeSeq for MarshalSerializeSeq<'w, 'en, E> {
    type Ok = ();
    type Error = MarshalError;

    fn serialize_element<T>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + Serialize,
    {
        Ok(value.serialize(MarshalSerializer::<E>(self.0.encode_element()?))?)
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(self.0.end()?)
    }
}

impl<'w, 'en, E: Encoder> serde::ser::SerializeTuple for MarshalSerializeTuple<'w, 'en, E> {
    type Ok = ();
    type Error = MarshalError;

    fn serialize_element<T>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + Serialize,
    {
        Ok(value.serialize(MarshalSerializer::<E>(self.0.encode_element()?))?)
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(self.0.end()?)
    }
}

impl<'w, 'en, E: Encoder> serde::ser::SerializeTupleStruct
    for MarshalSerializeTupleStruct<'w, 'en, E>
{
    type Ok = ();
    type Error = MarshalError;

    fn serialize_field<T>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + Serialize,
    {
        Ok(value.serialize(MarshalSerializer::<E>(self.0.encode_field()?))?)
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(self.0.end()?)
    }
}

impl<'w, 'en, E: Encoder> serde::ser::SerializeStruct for MarshalSerializeStruct<'w, 'en, E> {
    type Ok = ();
    type Error = MarshalError;

    fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + Serialize,
    {
        let mut e = self.0.encode_entry()?;
        e.encode_key()?.encode_str(key)?;
        value.serialize(MarshalSerializer::<E>(e.encode_value()?))?;
        e.end()?;
        Ok(())
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(self.0.end()?)
    }
}

impl<'w, 'en, E: Encoder> serde::ser::SerializeTupleVariant
    for MarshalSerializeTupleVariant<'w, 'en, E>
{
    type Ok = ();
    type Error = MarshalError;

    fn serialize_field<T>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + Serialize,
    {
        let field = self.encoder.tuple_encode_element(&mut self.fields)?;
        value.serialize(MarshalSerializer::<E>(AnySpecEncoder::new(
            self.encoder,
            field,
        )))?;
        Ok(())
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        self.encoder.tuple_end(self.fields)?;
        self.encoder.entry_end(self.entry)?;
        self.encoder.map_end(self.map)?;
        Ok(())
    }
}

impl<'w, 'en, E: Encoder> serde::ser::SerializeStructVariant
    for MarshalSerializeStructVariant<'w, 'en, E>
{
    type Ok = ();
    type Error = MarshalError;

    fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + Serialize,
    {
        let (key_encoder, value_encoder) = self.encoder.map_encode_element(&mut self.fields)?;
        self.encoder.encode_str(key_encoder, key)?;
        let (value_encoder, entry) = self.encoder.entry_encode_value(value_encoder)?;
        value.serialize(MarshalSerializer::<E>(AnySpecEncoder::new(
            self.encoder,
            value_encoder,
        )))?;
        self.encoder.entry_end(entry)?;
        Ok(())
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        self.encoder.map_end(self.fields)?;
        self.encoder.entry_end(self.entry)?;
        self.encoder.map_end(self.map)?;
        Ok(())
    }
}

impl<'w, 'en, E: Encoder> serde::ser::SerializeMap for MarshalSerializeMap<'w, 'en, E> {
    type Ok = ();
    type Error = MarshalError;

    fn serialize_key<T>(&mut self, key: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + Serialize,
    {
        let (key_encoder, value) = self.encoder.map_encode_element(&mut self.map)?;
        self.entry = Some(value);
        key.serialize(MarshalSerializer::<E>(AnySpecEncoder::new(
            self.encoder,
            key_encoder,
        )))?;
        Ok(())
    }

    fn serialize_value<T>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + Serialize,
    {
        let (value_encoder, entry) = self
            .encoder
            .entry_encode_value(self.entry.take().unwrap())?;
        value.serialize(MarshalSerializer::<E>(AnySpecEncoder::new(
            self.encoder,
            value_encoder,
        )))?;
        self.encoder.entry_end(entry)?;
        Ok(())
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        self.encoder.map_end(self.map)?;
        Ok(())
    }
}

impl<E: Encoder, T> marshal::ser::Serialize<E> for SerdeWrapper<T>
where
    T: serde::Serialize,
{
    fn serialize<'w, 'en>(&self, e: AnyEncoder<'w, 'en, E>, _ctx: Context) -> anyhow::Result<()> {
        <T as serde::Serialize>::serialize(&self.inner, MarshalSerializer::<E>(e)).map_err(|x| x.0)
    }
}
