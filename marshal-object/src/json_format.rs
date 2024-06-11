use std::marker::PhantomData;
use std::ops::CoerceUnsized;

use type_map::concurrent::TypeMap;

use crate::de::{DeserializeProvider, DeserializeVariant, DeserializeVariantProvider};
use marshal::context::Context;
use marshal::de::Deserialize;
use marshal::decode::Decoder;
use marshal_json::decode::full::JsonDecoder;
use marshal_json::DeserializeJson;
use marshal_json::SerializeJson;

use crate::{Object};

pub trait SerializeDyn = SerializeJson;

pub trait DeserializeVariantJson<O: Object>: 'static + Sync + Send {
    fn deserialize_variant_json<'p, 'de>(
        &self,
        d: <JsonDecoder<'de> as Decoder<'de>>::AnyDecoder<'p>,
        ctx: &mut Context,
    ) -> anyhow::Result<O::Pointer<O::Dyn>>;
}

impl<O: Object, V: 'static> DeserializeVariantJson<O> for PhantomData<fn() -> V>
where
    O::Pointer<V>: CoerceUnsized<O::Pointer<O::Dyn>> + for<'de> Deserialize<'de, JsonDecoder<'de>>,
{
    fn deserialize_variant_json<'p, 'de, 's>(
        &self,
        d: <JsonDecoder<'de> as Decoder<'de>>::AnyDecoder<'p>,
        ctx: &mut Context,
    ) -> anyhow::Result<O::Pointer<O::Dyn>> {
        Ok(O::Pointer::<V>::deserialize(d, ctx)?)
    }
}

pub struct FormatDeserializeProvider<O: Object>(PhantomData<O>);

impl<O: Object> DeserializeProvider for FormatDeserializeProvider<O> {}

impl<O: Object, V: 'static> DeserializeVariantProvider<V> for FormatDeserializeProvider<O>
where
    O::Pointer<V>: CoerceUnsized<O::Pointer<O::Dyn>> + for<'de> DeserializeJson<'de>,
{
    fn add_deserialize_variant(map: &mut TypeMap) {
        map.insert(Box::new(PhantomData::<fn() -> V>) as Box<dyn DeserializeVariantJson<O>>);
    }
}

impl<O: Object> DeserializeVariant for Box<dyn DeserializeVariantJson<O>> {}

#[macro_export]
macro_rules! json_format {
    ($carrier:path) => {
        impl<'de> $crate::de::DeserializeVariantForDiscriminant<'de, $crate::reexports::marshal_json::decode::full::JsonDecoder<'de>> for $carrier {
            fn deserialize_variant(
                disc: usize,
                d: <$crate::reexports::marshal_json::decode::full::JsonDecoder<'de> as $crate::reexports::marshal::decode::Decoder<'de>>::AnyDecoder<'_>,
                ctx: &mut $crate::reexports::marshal::context::Context,
            ) -> $crate::reexports::anyhow::Result<<$carrier as $crate::Object>::Pointer<<$carrier as $crate::Object>::Dyn>> {
                static DESERIALIZERS: LazyLock<$crate::de::DeserializeVariantTable<$carrier, ::std::boxed::Box<dyn $crate::json_format::DeserializeVariantJson<$carrier>>>> =
                    LazyLock::new($crate::de::DeserializeVariantTable::new);
                DESERIALIZERS[disc].deserialize_variant_json(d, ctx)
            }
        }
    };
}
