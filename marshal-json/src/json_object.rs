use std::marker::PhantomData;
use std::ops::CoerceUnsized;

use crate::decode::full::JsonDecoder;
use crate::SerializeJson;
use marshal::context::Context;
use marshal::de::Deserialize;
use marshal::decode::Decoder;
use marshal_core::encode::{Encoder};
use marshal_object::de::{
    DeserializeProvider, DeserializeVariant, DeserializeVariantProvider, DeserializeVariantSet,
};
use marshal_object::ser::DowncastSerialize;
use marshal_object::Object;

use crate::encode::full::JsonEncoder;
use crate::DeserializeJson;

pub trait SerializeDyn = SerializeJson;

pub trait DeserializeVariantJson<O: Object>: 'static + Sync + Send {
    fn deserialize_variant_json<'p, 'de>(
        &self,
        d: <JsonDecoder<'de> as Decoder<'de>>::AnyDecoder<'p>,
        ctx: &mut Context,
    ) -> anyhow::Result<O::Pointer<O::Dyn>>;
    fn serialize_variant_json<'p>(
        &self,
        this: &O::Pointer<O::Dyn>,
        e: <JsonEncoder as Encoder>::AnyEncoder<'p>,
        ctx: &mut Context,
    ) -> anyhow::Result<()>;
}

impl<O: Object, V: 'static> DeserializeVariantJson<O> for PhantomData<fn() -> V>
where
    O::Pointer<V>: CoerceUnsized<O::Pointer<O::Dyn>>,
    O::Pointer<V>: for<'de> Deserialize<'de, JsonDecoder<'de>>,
    O::Pointer<O::Dyn>: DowncastSerialize<V, JsonEncoder>,
{
    fn deserialize_variant_json<'p, 'de, 's>(
        &self,
        d: <JsonDecoder<'de> as Decoder<'de>>::AnyDecoder<'p>,
        ctx: &mut Context,
    ) -> anyhow::Result<O::Pointer<O::Dyn>> {
        Ok(O::Pointer::<V>::deserialize(d, ctx)?)
    }
    fn serialize_variant_json<'p>(
        &self,
        this: &O::Pointer<O::Dyn>,
        e: <JsonEncoder as Encoder>::AnyEncoder<'p>,
        ctx: &mut Context,
    ) -> anyhow::Result<()> {
        this.downcast_serialize(e, ctx)
    }
}

pub struct FormatDeserializeProvider<O: Object>(PhantomData<O>);

impl<O: Object> DeserializeProvider for FormatDeserializeProvider<O> {}

impl<O: Object, V: 'static> DeserializeVariantProvider<V> for FormatDeserializeProvider<O>
where
    O::Pointer<V>: CoerceUnsized<O::Pointer<O::Dyn>> + for<'de> DeserializeJson<'de>,
    O::Pointer<O::Dyn>: DowncastSerialize<V, JsonEncoder>,
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
            static DESERIALIZERS: LazyLock<$crate::reexports::marshal_object::de::DeserializeVariantTable<$carrier, ::std::boxed::Box<dyn $crate::json_object::DeserializeVariantJson<$carrier>>>> =
                    LazyLock::new($crate::reexports::marshal_object::de::DeserializeVariantTable::new);
            impl<'de> $crate::reexports::marshal_object::de::DeserializeVariantForDiscriminant<'de, $crate::decode::full::JsonDecoder<'de>> for $carrier {
                fn deserialize_variant(
                    disc: usize,
                    d: <$crate::decode::full::JsonDecoder<'de> as $crate::reexports::marshal::decode::Decoder<'de>>::AnyDecoder<'_>,
                    ctx: &mut $crate::reexports::marshal::context::Context,
                ) -> $crate::reexports::anyhow::Result<<$carrier as $crate::reexports::marshal_object::Object>::Pointer<<$carrier as $crate::reexports::marshal_object::Object>::Dyn>> {
                    DESERIALIZERS[disc].deserialize_variant_json(d, ctx)
                }
            }
            impl $crate::reexports::marshal_object::ser::SerializeVariantForDiscriminant<$crate::encode::full::JsonEncoder> for $carrier {
                fn serialize_variant(
                    this: &Self::Pointer<Self::Dyn>,
                    disc:usize,
                    e: <$crate::encode::full::JsonEncoder as $crate::reexports::marshal::encode::Encoder>::AnyEncoder<'_>,
                    ctx: &mut Context
                ) -> anyhow::Result<()> {
                    DESERIALIZERS[disc].serialize_variant_json(this, e, ctx)
                }
            }
        };
    };
}
