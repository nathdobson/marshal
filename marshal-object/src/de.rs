use marshal::context::Context;
use marshal::de::SchemaError;
use marshal::decode::{
    AnyDecoder, DecodeHint, DecodeVariantHint, Decoder, DecoderView, EnumDecoder, SeqDecoder,
};
use std::ops::{CoerceUnsized, Deref};
use type_map::concurrent::TypeMap;

use crate::Object;

pub trait DeserializeVariant<'de, D: Decoder<'de>, P: Deref> {
    fn deserialize_variant(
        discriminant: usize,
        d: D::AnyDecoder<'_>,
        ctx: &mut Context,
    ) -> anyhow::Result<P>;
}

pub fn deserialize_object<
    'p,
    'de,
    D: Decoder<'de>,
    T: ?Sized + Object + DeserializeVariant<'de, D, P>,
    P: Deref,
>(
    d: D::AnyDecoder<'p>,
    ctx: &mut Context,
) -> anyhow::Result<P> {
    match d.decode(DecodeHint::Enum {
        name: T::object_descriptor().object_name(),
        variants: T::object_descriptor().discriminant_names(),
    })? {
        DecoderView::Enum(mut d) => {
            let disc = match d.decode_discriminant()?.decode(DecodeHint::Identifier)? {
                DecoderView::String(x) => T::object_descriptor()
                    .variant_index_of(&*x)
                    .ok_or(SchemaError::UnknownVariant)?,
                DecoderView::Primitive(p) => p.try_into()?,
                d => d.mismatch("discriminant")?,
            };
            let result = match d.decode_variant(DecodeVariantHint::TupleVariant { len: 1 })? {
                DecoderView::Seq(mut d) => {
                    let variant = d.decode_next()?.ok_or(SchemaError::TupleTooShort)?;
                    let result =
                        <T as DeserializeVariant<D, P>>::deserialize_variant(disc, variant, ctx)?;
                    if let Some(_) = d.decode_next()? {
                        return Err(SchemaError::TupleTooLong.into());
                    }
                    result
                }
                d => d.mismatch("seq")?,
            };
            d.decode_end()?;
            Ok(result)
        }
        d => d.mismatch("enum")?,
    }
}

pub trait Format {}

pub trait VariantFormat<V: 'static + Deref>: Format {
    fn add_object_deserializer<T: 'static + Deref>(map: &mut TypeMap)
    where
        V: CoerceUnsized<T>;
}
