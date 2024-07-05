use std::marker::{PhantomData, Unsize};
use std::rc;

use marshal::context::Context;
use marshal::encode::{AnyEncoder, Encoder};
use marshal::ser::Serialize;
use marshal_pointer::AsFlatRef;
use marshal_pointer::raw_any::{DowncastRef, RawAny};

use crate::{AsDiscriminant, Object};
use crate::variants::{VariantImpl, VariantImplSet};

pub fn serialize_object<'w, 'en, O: Object, E: Encoder>(
    value: &<O::Pointer<O::Dyn> as AsFlatRef>::FlatRef,
    e: AnyEncoder<'w, 'en, E>,
    ctx: Context,
) -> anyhow::Result<()>
where
    O: SerializeVariantForDiscriminant<E>,
{
    let disc = O::discriminant_of(value);
    let mut e = e.encode_tuple_variant(
        O::object_descriptor().object_name(),
        O::object_descriptor().discriminant_names(),
        disc,
        1,
    )?;
    O::serialize_variant(value, disc, e.encode_field()?, ctx)?;
    e.end()?;
    Ok(())
}

pub fn serialize_rc_weak_object<'w, 'en, O: Object, E: Encoder>(
    value: &rc::Weak<O::Dyn>,
    e: AnyEncoder<'w, 'en, E>,
    ctx: Context,
) -> anyhow::Result<()>
where
    O::Dyn: Serialize<E>,
{
    let mut e = e.encode_tuple_variant(
        O::object_descriptor().object_name(),
        O::object_descriptor().discriminant_names(),
        value.as_ptr().as_discriminant(),
        1,
    )?;
    if let Some(value) = value.upgrade() {
        let mut e = e.encode_field()?.encode_some()?;
        <O::Dyn as Serialize<E>>::serialize(&value, e.encode_some()?, ctx)?;
        e.end()?;
    } else {
        e.encode_field()?.encode_none()?;
    }
    e.end()?;
    Ok(())
}

pub trait SerializeVariantForDiscriminant<E: Encoder>: Object {
    fn serialize_variant<'w, 'en>(
        this: &<Self::Pointer<Self::Dyn> as AsFlatRef>::FlatRef,
        disc: usize,
        e: AnyEncoder<'w, 'en, E>,
        ctx: Context,
    ) -> anyhow::Result<()>;
}

pub trait SerializeVariantDyn<E: Encoder, O: Object>: Sync + Send {
    fn serialize_variant_dyn<'w, 'en>(
        &self,
        this: &<O::Pointer<O::Dyn> as AsFlatRef>::FlatRef,
        e: AnyEncoder<'w, 'en, E>,
        ctx: Context,
    ) -> anyhow::Result<()>;
}

impl<E: Encoder, O: Object, V> SerializeVariantDyn<E, O> for PhantomData<fn() -> V>
where
    <O::Pointer<dyn RawAny> as AsFlatRef>::FlatRef:
        DowncastRef<<O::Pointer<V> as AsFlatRef>::FlatRef>,
    <O::Pointer<V> as AsFlatRef>::FlatRef: Serialize<E>,
    <O::Pointer<O::Dyn> as AsFlatRef>::FlatRef:
        Unsize<<O::Pointer<dyn RawAny> as AsFlatRef>::FlatRef>,
{
    fn serialize_variant_dyn<'w, 'en>(
        &self,
        this: &<O::Pointer<O::Dyn> as AsFlatRef>::FlatRef,
        e: AnyEncoder<'w, 'en, E>,
        ctx: Context,
    ) -> anyhow::Result<()> {
        let upcast = this as &<O::Pointer<dyn RawAny> as AsFlatRef>::FlatRef;
        let downcast: &<O::Pointer<V> as AsFlatRef>::FlatRef = upcast
            .downcast_ref()
            .expect("failed to downcast for serializer");
        <<O::Pointer<V> as AsFlatRef>::FlatRef as Serialize<E>>::serialize(downcast, e, ctx)
    }
}

impl<E, O> VariantImpl for &'static dyn SerializeVariantDyn<E, O> {}

pub trait SerializeProvider<V: 'static> {
    fn add_serialize_variant(map: &mut VariantImplSet);
}
