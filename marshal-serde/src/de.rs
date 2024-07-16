use std::borrow::Cow;

use serde::de::{DeserializeSeed, EnumAccess, MapAccess, SeqAccess, VariantAccess, Visitor};
use serde::Deserializer;

use marshal::context::Context;
use marshal::decode::{
    AnyDecoder, AnySpecDecoder, DecodeHint, Decoder, DecoderView, EntryDecoder, SeqDecoder,
    SpecDecoder,
};
use marshal::{Primitive, PrimitiveType, SchemaError};

use crate::{MarshalError, WithSerde};

struct MarshalDeserializer<'p, 'de, D: Decoder>(AnyDecoder<'p, 'de, D>);

struct MarshalSeqAccess<'p2, 'p, 'de, D: Decoder>(
    &'p2 mut SeqDecoder<'p, 'de, D::SpecDecoder<'de>>,
);
struct MarshalMapAccess<'p, 'de, D: Decoder> {
    decoder: &'p mut D::SpecDecoder<'de>,
    entry: Option<<D::SpecDecoder<'de> as SpecDecoder<'de>>::ValueDecoder>,
    map: Option<<D::SpecDecoder<'de> as SpecDecoder<'de>>::MapDecoder>,
}

struct MarshalEnumAccess<'p, 'de, D: Decoder>(EntryDecoder<'p, 'de, D::SpecDecoder<'de>>);
struct MarshalVariantAccess<'p, 'de, D: Decoder>(EntryDecoder<'p, 'de, D::SpecDecoder<'de>>);

impl<D: Decoder, T: for<'de> serde::Deserialize<'de>> marshal::de::Deserialize<D> for WithSerde<T> {
    fn deserialize<'p, 'de>(d: AnyDecoder<'p, 'de, D>, _ctx: Context) -> anyhow::Result<Self>
    where
        Self: Sized,
    {
        Ok(WithSerde {
            inner: T::deserialize(MarshalDeserializer::<D>(d)).map_err(|x| x.0)?,
        })
    }
}

fn visit<'p, 'de, D: Decoder, V: Visitor<'de>>(
    visitor: V,
    any: DecoderView<'p, 'de, D::SpecDecoder<'de>>,
) -> Result<V::Value, MarshalError> {
    match any {
        DecoderView::Primitive(p) => match p {
            Primitive::Unit => visitor.visit_unit(),
            Primitive::Bool(x) => visitor.visit_bool(x),
            Primitive::I8(x) => visitor.visit_i8(x),
            Primitive::I16(x) => visitor.visit_i16(x),
            Primitive::I32(x) => visitor.visit_i32(x),
            Primitive::I64(x) => visitor.visit_i64(x),
            Primitive::I128(x) => visitor.visit_i128(x),
            Primitive::U8(x) => visitor.visit_u8(x),
            Primitive::U16(x) => visitor.visit_u16(x),
            Primitive::U32(x) => visitor.visit_u32(x),
            Primitive::U64(x) => visitor.visit_u64(x),
            Primitive::U128(x) => visitor.visit_u128(x),
            Primitive::F32(x) => visitor.visit_f32(x),
            Primitive::F64(x) => visitor.visit_f64(x),
            Primitive::Char(x) => visitor.visit_char(x),
        },
        DecoderView::String(x) => match x {
            Cow::Borrowed(x) => visitor.visit_borrowed_str(x),
            Cow::Owned(x) => visitor.visit_string(x),
        },
        DecoderView::Bytes(x) => match x {
            Cow::Borrowed(x) => visitor.visit_borrowed_bytes(x),
            Cow::Owned(x) => visitor.visit_byte_buf(x),
        },
        DecoderView::None => visitor.visit_none(),
        DecoderView::Some(mut d) => {
            let result = visitor.visit_some(MarshalDeserializer::<D>(d.decode_some()?))?;
            d.decode_end()?;
            Ok(result)
        }
        DecoderView::Seq(mut d) => {
            let result = visitor.visit_seq(MarshalSeqAccess::<D>(&mut d))?;
            d.ignore()?;
            Ok(result)
        }
        DecoderView::Map(d) => {
            let (decoder, map) = d.into_raw();
            let result = visitor.visit_map(MarshalMapAccess::<D> {
                decoder,
                entry: None,
                map: Some(map),
            })?;
            Ok(result)
        }
        DecoderView::Enum(_) => todo!(),
    }
}

impl<'p, 'de, D: Decoder> Deserializer<'de> for MarshalDeserializer<'p, 'de, D> {
    type Error = MarshalError;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visit::<D, V>(visitor, self.0.decode(DecodeHint::Any)?)
    }

    fn deserialize_bool<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visit::<D, V>(
            visitor,
            self.0.decode(DecodeHint::Primitive(PrimitiveType::Bool))?,
        )
    }

    fn deserialize_i8<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visit::<D, V>(
            visitor,
            self.0.decode(DecodeHint::Primitive(PrimitiveType::I8))?,
        )
    }

    fn deserialize_i16<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visit::<D, V>(
            visitor,
            self.0.decode(DecodeHint::Primitive(PrimitiveType::I16))?,
        )
    }

    fn deserialize_i32<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visit::<D, V>(
            visitor,
            self.0.decode(DecodeHint::Primitive(PrimitiveType::I32))?,
        )
    }

    fn deserialize_i64<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visit::<D, V>(
            visitor,
            self.0.decode(DecodeHint::Primitive(PrimitiveType::I64))?,
        )
    }

    fn deserialize_i128<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visit::<D, V>(
            visitor,
            self.0.decode(DecodeHint::Primitive(PrimitiveType::I128))?,
        )
    }

    fn deserialize_u8<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visit::<D, V>(
            visitor,
            self.0.decode(DecodeHint::Primitive(PrimitiveType::U8))?,
        )
    }

    fn deserialize_u16<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visit::<D, V>(
            visitor,
            self.0.decode(DecodeHint::Primitive(PrimitiveType::U16))?,
        )
    }

    fn deserialize_u32<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visit::<D, V>(
            visitor,
            self.0.decode(DecodeHint::Primitive(PrimitiveType::U32))?,
        )
    }

    fn deserialize_u64<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visit::<D, V>(
            visitor,
            self.0.decode(DecodeHint::Primitive(PrimitiveType::U64))?,
        )
    }

    fn deserialize_u128<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visit::<D, V>(
            visitor,
            self.0.decode(DecodeHint::Primitive(PrimitiveType::U128))?,
        )
    }

    fn deserialize_f32<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visit::<D, V>(
            visitor,
            self.0.decode(DecodeHint::Primitive(PrimitiveType::F32))?,
        )
    }

    fn deserialize_f64<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visit::<D, V>(
            visitor,
            self.0.decode(DecodeHint::Primitive(PrimitiveType::F64))?,
        )
    }

    fn deserialize_char<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visit::<D, V>(
            visitor,
            self.0.decode(DecodeHint::Primitive(PrimitiveType::Char))?,
        )
    }

    fn deserialize_str<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visit::<D, V>(visitor, self.0.decode(DecodeHint::String)?)
    }

    fn deserialize_string<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visit::<D, V>(visitor, self.0.decode(DecodeHint::String)?)
    }

    fn deserialize_bytes<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visit::<D, V>(visitor, self.0.decode(DecodeHint::Bytes)?)
    }

    fn deserialize_byte_buf<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visit::<D, V>(visitor, self.0.decode(DecodeHint::Bytes)?)
    }

    fn deserialize_option<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visit::<D, V>(visitor, self.0.decode(DecodeHint::Option)?)
    }

    fn deserialize_unit<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visit::<D, V>(
            visitor,
            self.0.decode(DecodeHint::Primitive(PrimitiveType::Unit))?,
        )
    }

    fn deserialize_unit_struct<V>(
        self,
        name: &'static str,
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visit::<D, V>(visitor, self.0.decode(DecodeHint::UnitStruct { name })?)
    }

    fn deserialize_newtype_struct<V>(
        self,
        name: &'static str,
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visit::<D, V>(
            visitor,
            self.0.decode(DecodeHint::TupleStruct { name, len: 1 })?,
        )
    }

    fn deserialize_seq<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visit::<D, V>(visitor, self.0.decode(DecodeHint::Seq)?)
    }

    fn deserialize_tuple<V>(self, len: usize, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visit::<D, V>(visitor, self.0.decode(DecodeHint::Tuple { len })?)
    }

    fn deserialize_tuple_struct<V>(
        self,
        name: &'static str,
        len: usize,
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visit::<D, V>(
            visitor,
            self.0.decode(DecodeHint::TupleStruct { name, len })?,
        )
    }

    fn deserialize_map<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visit::<D, V>(visitor, self.0.decode(DecodeHint::Map)?)
    }

    fn deserialize_struct<V>(
        self,
        name: &'static str,
        fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visit::<D, V>(visitor, self.0.decode(DecodeHint::Struct { name, fields })?)
    }

    fn deserialize_enum<V>(
        self,
        _name: &'static str,
        _variants: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        let mut d = self.0.decode(DecodeHint::Map)?.try_into_map()?;
        let output;
        {
            let d = d
                .decode_next()?
                .ok_or_else(|| anyhow::Error::from(SchemaError::TupleTooShort))?;
            output = visitor.visit_enum(MarshalEnumAccess::<D>(d))?;
        }
        if let Some(_) = d.decode_next()? {
            return Err(anyhow::Error::from(SchemaError::TupleTooLong { expected: 1 }).into());
        }
        Ok(output)
    }

    fn deserialize_identifier<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visit::<D, V>(visitor, self.0.decode(DecodeHint::Identifier)?)
    }

    fn deserialize_ignored_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visit::<D, V>(visitor, self.0.decode(DecodeHint::Ignore)?)
    }

    fn is_human_readable(&self) -> bool {
        self.0.is_human_readable()
    }
}

impl<'p2, 'p, 'de, D: Decoder> SeqAccess<'de> for MarshalSeqAccess<'p2, 'p, 'de, D> {
    type Error = MarshalError;

    fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>, Self::Error>
    where
        T: DeserializeSeed<'de>,
    {
        if let Some(d) = self.0.decode_next()? {
            Ok(Some(seed.deserialize(MarshalDeserializer::<D>(d))?))
        } else {
            Ok(None)
        }
    }
}

impl<'p, 'de, D: Decoder> MapAccess<'de> for MarshalMapAccess<'p, 'de, D> {
    type Error = MarshalError;

    fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>, Self::Error>
    where
        K: DeserializeSeed<'de>,
    {
        if let Some(d) = self.decoder.decode_map_next(self.map.as_mut().unwrap())? {
            let (key, value) = self.decoder.decode_entry_key(d)?;
            let key = seed.deserialize(MarshalDeserializer::<D>(AnySpecDecoder::new(
                self.decoder,
                key,
            )))?;
            self.entry = Some(value);
            Ok(Some(key))
        } else {
            self.decoder.decode_map_end(self.map.take().unwrap())?;
            Ok(None)
        }
    }

    fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value, Self::Error>
    where
        V: DeserializeSeed<'de>,
    {
        let value = self
            .decoder
            .decode_entry_value(self.entry.take().unwrap())?;
        let result = seed.deserialize(MarshalDeserializer::<D>(AnySpecDecoder::new(
            self.decoder,
            value,
        )))?;
        Ok(result)
    }
}

impl<'p, 'de, D: Decoder> EnumAccess<'de> for MarshalEnumAccess<'p, 'de, D> {
    type Error = MarshalError;
    type Variant = MarshalVariantAccess<'p, 'de, D>;

    fn variant_seed<V>(mut self, seed: V) -> Result<(V::Value, Self::Variant), Self::Error>
    where
        V: DeserializeSeed<'de>,
    {
        let result = seed.deserialize(MarshalDeserializer::<D>(self.0.decode_key()?))?;
        Ok((result, MarshalVariantAccess(self.0)))
    }
}

impl<'p, 'de, D: Decoder> VariantAccess<'de> for MarshalVariantAccess<'p, 'de, D> {
    type Error = MarshalError;

    fn unit_variant(mut self) -> Result<(), Self::Error> {
        self.0
            .decode_value()?
            .decode(DecodeHint::Primitive(PrimitiveType::Unit))?
            .try_into_unit()?;
        self.0.decode_end()?;
        Ok(())
    }

    fn newtype_variant_seed<T>(mut self, seed: T) -> Result<T::Value, Self::Error>
    where
        T: DeserializeSeed<'de>,
    {
        let mut seq = self
            .0
            .decode_value()?
            .decode(DecodeHint::Tuple { len: 1 })?
            .try_into_seq()?;
        let result = seed.deserialize(MarshalDeserializer::<D>(
            seq.decode_next()?
                .ok_or(SchemaError::TupleTooShort)
                .map_err(anyhow::Error::from)?,
        ))?;
        if let Some(_) = seq.decode_next()? {
            return Err(MarshalError(anyhow::Error::from(
                SchemaError::TupleTooLong { expected: 1 },
            )));
        }
        self.0.decode_end()?;
        Ok(result)
    }

    fn tuple_variant<V>(mut self, len: usize, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        let result = visit::<D, V>(
            visitor,
            self.0.decode_value()?.decode(DecodeHint::Tuple { len })?,
        )?;
        self.0.decode_end()?;
        Ok(result)
    }

    fn struct_variant<V>(
        mut self,
        _fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        let result = visit::<D, V>(visitor, self.0.decode_value()?.decode(DecodeHint::Map)?)?;
        self.0.decode_end()?;
        Ok(result)
    }
}
