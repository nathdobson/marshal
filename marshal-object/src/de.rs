use std::marker::PhantomData;
use std::ops::{CoerceUnsized};

use marshal::context::Context;
use marshal::de::{Deserialize, SchemaError};
use marshal::decode::{AnyDecoder, DecodeHint, DecodeVariantHint, Decoder, DecoderView};

use crate::{Object};
use crate::variants::VariantImpl;

pub trait DeserializeVariantForDiscriminant<D: Decoder>: Object {
    fn deserialize_variant<'p, 'de>(
        discriminant: usize,
        d: AnyDecoder<'p, 'de, D>,
        ctx: Context,
    ) -> anyhow::Result<Self::Pointer<Self::Dyn>>;
}

pub fn deserialize_object<'p, 'de, O: Object, D: Decoder>(
    d: AnyDecoder<'p, 'de, D>,
    ctx: Context,
) -> anyhow::Result<O::Pointer<O::Dyn>>
where
    O: DeserializeVariantForDiscriminant<D>,
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
                    let result = O::deserialize_variant(disc, variant, ctx)?;
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



pub trait DeserializeVariantDyn<D: Decoder, O: Object>: 'static + Sync + Send {
    fn deserialize_variant_dyn<'p, 'de>(
        &self,
        d: AnyDecoder<'p, 'de, D>,
        ctx: Context,
    ) -> anyhow::Result<O::Pointer<O::Dyn>>;
}

// pub struct DeserializeVariantDynValue<V>();
//
// impl<V> DeserializeVariantDynValue<V> {
//     pub const fn new() -> Self {
//         DeserializeVariantDynValue(PhantomData)
//     }
// }

impl<D, O> VariantImpl for &'static dyn DeserializeVariantDyn<D, O> {}

impl<D: Decoder, O: Object, V: 'static> DeserializeVariantDyn<D, O>
    for PhantomData<fn() -> V>
where
    O::Pointer<V>: Deserialize<D>,
    O::Pointer<V>: CoerceUnsized<O::Pointer<O::Dyn>>,
{
    fn deserialize_variant_dyn<'p, 'de>(
        &self,
        d: AnyDecoder<'p, 'de, D>,
        ctx: Context,
    ) -> anyhow::Result<O::Pointer<O::Dyn>> {
        Ok(<O::Pointer<V> as Deserialize<D>>::deserialize(d, ctx)?)
    }
}
