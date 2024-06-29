use std::marker::{PhantomData, Unsize};
use std::ops::CoerceUnsized;

use marshal::context::Context;
use marshal::de::Deserialize;
use marshal::ser::Serialize;
use marshal_core::decode::AnyDecoder;
use marshal_core::encode::AnyEncoder;
use marshal_object::de::{
    DeserializeProvider, DeserializeVariant, DeserializeVariantProvider, DeserializeVariantSet,
};
// use marshal_object::ser::DowncastSerialize;
use marshal_object::Object;
use marshal_pointer::{AsFlatRef, DowncastRef, RawAny};

use crate::decode::full::{JsonDecoder, JsonGenDecoder};
use crate::encode::full::{JsonEncoder, JsonGenEncoder};
use crate::DeserializeJson;
use crate::SerializeJson;

pub trait SerializeDyn = SerializeJson;

pub trait DeserializeVariantJson<O: Object>: 'static + Sync + Send {
    fn deserialize_variant_json<'p, 'de>(
        &self,
        d: AnyDecoder<'p, 'de, JsonDecoder<'de>>,
        ctx: Context,
    ) -> anyhow::Result<O::Pointer<O::Dyn>>;
    fn serialize_variant_json<'p>(
        &self,
        this: &<O::Pointer<O::Dyn> as AsFlatRef>::FlatRef,
        e: AnyEncoder<'p, JsonEncoder>,
        ctx: Context,
    ) -> anyhow::Result<()>;
}

impl<O: Object, V: 'static> DeserializeVariantJson<O> for PhantomData<fn() -> V>
where
    O::Pointer<V>: CoerceUnsized<O::Pointer<O::Dyn>>,
    O::Pointer<V>: Deserialize<JsonGenDecoder>,
    <O::Pointer<O::Dyn> as AsFlatRef>::FlatRef:
        Unsize<<O::Pointer<dyn RawAny> as AsFlatRef>::FlatRef>,
    <O::Pointer<dyn RawAny> as AsFlatRef>::FlatRef:
        DowncastRef<<O::Pointer<V> as AsFlatRef>::FlatRef>,
    <O::Pointer<V> as AsFlatRef>::FlatRef: SerializeJson,
{
    fn deserialize_variant_json<'p, 'de, 's>(
        &self,
        d: AnyDecoder<'p, 'de, JsonDecoder<'de>>,
        mut ctx: Context,
    ) -> anyhow::Result<O::Pointer<O::Dyn>> {
        Ok(O::Pointer::<V>::deserialize(d, ctx)?)
    }
    fn serialize_variant_json<'p>(
        &self,
        this: &<O::Pointer<O::Dyn> as AsFlatRef>::FlatRef,
        e: AnyEncoder<'p, JsonEncoder>,
        mut ctx: Context,
    ) -> anyhow::Result<()> {
        let upcast = this as &<O::Pointer<dyn RawAny> as AsFlatRef>::FlatRef;
        let downcast = upcast
            .downcast_ref()
            .expect("failed to downcast for serializer");
        <<O::Pointer<V> as AsFlatRef>::FlatRef as Serialize<JsonGenEncoder>>::serialize(
            downcast, e, ctx,
        )
    }
}

pub struct FormatDeserializeProvider<O: Object>(PhantomData<O>);

impl<O: Object> DeserializeProvider for FormatDeserializeProvider<O> {}

impl<O: Object, V: 'static> DeserializeVariantProvider<V> for FormatDeserializeProvider<O>
where
    O::Pointer<V>: CoerceUnsized<O::Pointer<O::Dyn>> + DeserializeJson,
    <O::Pointer<V> as AsFlatRef>::FlatRef: SerializeJson,
    <O::Pointer<O::Dyn> as AsFlatRef>::FlatRef:
        Unsize<<O::Pointer<dyn RawAny> as AsFlatRef>::FlatRef>,
    <O::Pointer<dyn RawAny> as AsFlatRef>::FlatRef:
        DowncastRef<<O::Pointer<V> as AsFlatRef>::FlatRef>,
    <O::Pointer<V> as AsFlatRef>::FlatRef: SerializeJson,
{
    fn add_deserialize_variant(map: &mut DeserializeVariantSet) {
        map.insert(Box::new(PhantomData::<fn() -> V>) as Box<dyn DeserializeVariantJson<O>>);
    }
}

impl<O: Object> DeserializeVariant for Box<dyn DeserializeVariantJson<O>> {}

#[macro_export]
macro_rules! json_object {
    ($carrier:path) => {
        const _: () = {
            static DESERIALIZERS: $crate::reexports::safe_once::sync::LazyLock<$crate::reexports::marshal_object::de::DeserializeVariantTable<$carrier, ::std::boxed::Box<dyn $crate::json_object::DeserializeVariantJson<$carrier>>>> =
                    $crate::reexports::safe_once::sync::LazyLock::new($crate::reexports::marshal_object::de::DeserializeVariantTable::new);
            impl $crate::reexports::marshal_object::de::DeserializeVariantForDiscriminant<$crate::decode::full::JsonGenDecoder> for $carrier {
                fn deserialize_variant<'p, 'de>(
                    disc: usize,
                    d: $crate::reexports::marshal::decode::AnyDecoder<'p,'de,$crate::decode::full::JsonDecoder<'de>>,
                    mut ctx: $crate::reexports::marshal::context::Context,
                ) -> $crate::reexports::anyhow::Result<<$carrier as $crate::reexports::marshal_object::Object>::Pointer<<$carrier as $crate::reexports::marshal_object::Object>::Dyn>> {
                    DESERIALIZERS[disc].deserialize_variant_json(d, ctx)
                }
            }
            impl $crate::reexports::marshal_object::ser::SerializeVariantForDiscriminant<$crate::encode::full::JsonGenEncoder> for $carrier {
                fn serialize_variant<'w,'en>(
                    this: &<Self::Pointer<Self::Dyn> as $crate::reexports::marshal_pointer::AsFlatRef>::FlatRef,
                    disc:usize,
                    e: $crate::reexports::marshal::encode::AnyGenEncoder<'w,'en,$crate::encode::full::JsonGenEncoder>,
                    mut ctx: $crate::reexports::marshal::context::Context
                ) -> $crate::reexports::anyhow::Result<()> {
                    DESERIALIZERS[disc].serialize_variant_json(this, e, ctx)
                }
            }
        };
    };
}
