use crate::{Object, OBJECT_REGISTRY};
use marshal::context::Context;
use marshal::de::SchemaError;
use marshal::decode::{
    AnyDecoder, DecodeHint, DecodeVariantHint, Decoder, DecoderView, EnumDecoder, SeqDecoder,
};
use std::marker::PhantomData;
use std::ops::Index;
use type_map::concurrent::TypeMap;

pub trait DeserializeVariantForDiscriminant<'de, D: Decoder<'de>>: Object {
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
    O: DeserializeVariantForDiscriminant<'de, D>,
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

pub fn deserialize_rc_weak_object<'p, 'de, O: Object, D: Decoder<'de>>(
    d: D::AnyDecoder<'p>,
    ctx: &mut Context,
) -> anyhow::Result<O::Pointer<O::Dyn>>
where
    O: DeserializeVariantForDiscriminant<'de, D>,
{
    todo!();
}

pub trait DeserializeProvider {}

pub trait DeserializeVariantProvider<V: 'static>: DeserializeProvider {
    fn add_deserialize_variant(map: &mut TypeMap);
}

pub struct DeserializeVariantTable<O: Object, DV: DeserializeVariant> {
    variants: Vec<&'static DV>,
    phantom: PhantomData<O>,
}

pub trait DeserializeVariant: 'static + Sync + Send {}

impl<O: Object, DV: DeserializeVariant> DeserializeVariantTable<O, DV> {
    pub fn new() -> Self {
        let object = OBJECT_REGISTRY.object_descriptor::<O>();
        DeserializeVariantTable {
            variants: (0..object.discriminant_names().len())
                .map(|i| object.deserialize_variant(i))
                .collect(),
            phantom: PhantomData,
        }
    }
}

impl<O: Object, DV: DeserializeVariant> Index<usize> for DeserializeVariantTable<O, DV> {
    type Output = &'static DV;
    fn index(&self, index: usize) -> &Self::Output {
        &self.variants[index]
    }
}

pub struct DeserializeVariantSet(TypeMap);

impl DeserializeVariantSet {
    fn new() -> Self {
        DeserializeVariantSet(TypeMap::new())
    }
    pub fn insert<DV: DeserializeVariant>(&mut self, dv: DV) {
        self.0.insert(dv);
    }
    pub fn get<DV: DeserializeVariant>(&self) -> Option<&DV> {
        self.0.get::<DV>()
    }
}
