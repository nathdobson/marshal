use crate::Object;
use marshal::context::Context;
use marshal::de::SchemaError;
use marshal::decode::{
    AnyDecoder, DecodeHint, DecodeVariantHint, Decoder, DecoderView, EnumDecoder, SeqDecoder,
};
use type_map::concurrent::TypeMap;

pub trait DeserializeVariant<'de, D: Decoder<'de>>: Object {
    fn deserialize_variant(
        discriminant: usize,
        d: D::AnyDecoder<'_>,
        ctx: &mut Context,
    ) -> anyhow::Result<Self::Pointer<Self::Dyn>>;
}

pub fn deserialize_object<'p, 'de, O: Object, D: Decoder<'de>>(
    d: D::AnyDecoder<'p>,
    ctx: &mut Context,
) -> anyhow::Result<O::Pointer<O::Dyn>>
where
    O: DeserializeVariant<'de,D>,
{
    match d.decode(DecodeHint::Enum {
        name: O::object_descriptor().object_name(),
        variants: O::object_descriptor().discriminant_names(),
    })? {
        DecoderView::Enum(mut d) => {
            let disc = match d.decode_discriminant()?.decode(DecodeHint::Identifier)? {
                DecoderView::String(x) => O::object_descriptor()
                    .variant_index_of(&*x)
                    .ok_or(SchemaError::UnknownVariant)?,
                DecoderView::Primitive(p) => p.try_into()?,
                d => d.mismatch("discriminant")?,
            };
            let result = match d.decode_variant(DecodeVariantHint::TupleVariant { len: 1 })? {
                DecoderView::Seq(mut d) => {
                    let variant = d.decode_next()?.ok_or(SchemaError::TupleTooShort)?;
                    let result =
                        O::deserialize_variant(disc, variant, ctx)?;
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

pub trait VariantFormat<V: 'static>: Format {
    fn add_object_deserializer(map: &mut TypeMap);
}
