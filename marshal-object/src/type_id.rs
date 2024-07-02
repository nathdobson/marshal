use std::cmp::Ordering;
use std::hash::{Hash, Hasher};
use std::marker::{PhantomData, Unsize};
use std::ptr::null;

use marshal::context::Context;
use marshal::de::{Deserialize, SchemaError};
use marshal::decode::{AnyDecoder, DecodeHint, Decoder, DecodeVariantHint};
use marshal::encode::{AnyEncoder, Encoder};
use marshal::ser::Serialize;

use crate::{AsDiscriminant, Object};

pub struct ObjectTypeId<C> {
    discriminant: usize,
    phantom: PhantomData<fn() -> C>,
}

impl<C> Clone for ObjectTypeId<C> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<C> Copy for ObjectTypeId<C> {}

impl<C: Object> ObjectTypeId<C> {
    pub fn of<T>() -> Self
    where
        T: Unsize<C::Dyn>,
    {
        Self::of_dyn(null::<T>())
    }
    pub fn of_dyn(ptr: *const C::Dyn) -> Self {
        ObjectTypeId {
            discriminant: ptr.as_discriminant(),
            phantom: PhantomData,
        }
    }
}

impl<C: Object> Hash for ObjectTypeId<C> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.discriminant.hash(state)
    }
}

impl<C: Object> PartialEq<Self> for ObjectTypeId<C> {
    fn eq(&self, other: &Self) -> bool {
        self.discriminant == other.discriminant
    }
}

impl<C: Object> Eq for ObjectTypeId<C> {}

impl<C: Object> PartialOrd for ObjectTypeId<C> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.discriminant.partial_cmp(&other.discriminant)
    }
}

impl<C: Object> Ord for ObjectTypeId<C> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.discriminant.cmp(&other.discriminant)
    }
}

impl<E: Encoder, C: Object> Serialize<E> for ObjectTypeId<C> {
    fn serialize<'w, 'en>(&self, e: AnyEncoder<'w, 'en, E>, _ctx: Context) -> anyhow::Result<()> {
        e.encode_unit_variant(
            "ObjectTypeId",
            C::object_descriptor().discriminant_names(),
            self.discriminant,
        )
    }
}

impl<D: Decoder, C: Object> Deserialize<D> for ObjectTypeId<C> {
    fn deserialize<'p, 'de>(d: AnyDecoder<'p, 'de, D>, _ctx: Context) -> anyhow::Result<Self> {
        let variants = C::object_descriptor().discriminant_names();
        let mut d = d
            .decode(DecodeHint::Enum {
                name: "ObjectTypeId",
                variants,
            })?
            .try_into_enum()?;
        let discriminant = d
            .decode_discriminant()?
            .decode(DecodeHint::Identifier)?
            .try_into_identifier(variants)?
            .ok_or(SchemaError::UnknownVariant)?;
        d.decode_variant(DecodeVariantHint::UnitVariant)?.try_into_unit()?;
        d.decode_end()?;
        Ok(ObjectTypeId {
            discriminant,
            phantom: PhantomData,
        })
    }
}
