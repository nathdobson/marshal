use std::marker::PhantomData;
use std::ops::CoerceUnsized;

use type_map::concurrent::TypeMap;

use crate::de::{Format, VariantFormat};
use marshal::context::Context;
use marshal::de::Deserialize;
use marshal::decode::Decoder;
use marshal_json::decode::full::JsonDecoder;
use marshal_json::DeserializeJson;
use marshal_json::SerializeJson;

use crate::{Object, OBJECT_REGISTRY};

pub trait SerializeDyn = SerializeJson;

pub trait JsonObjectDeserializer<O: Object>: 'static + Sync + Send {
    fn json_object_deserialize<'p, 'de>(
        &self,
        d: <JsonDecoder<'de> as Decoder<'de>>::AnyDecoder<'p>,
        ctx: &mut Context,
    ) -> anyhow::Result<O::Pointer<O::Dyn>>;
}

impl<O: Object, V: 'static> JsonObjectDeserializer<O> for PhantomData<fn() -> V>
where
    O::Pointer<V>: CoerceUnsized<O::Pointer<O::Dyn>> + for<'de> Deserialize<'de, JsonDecoder<'de>>,
{
    fn json_object_deserialize<'p, 'de, 's>(
        &self,
        d: <JsonDecoder<'de> as Decoder<'de>>::AnyDecoder<'p>,
        ctx: &mut Context,
    ) -> anyhow::Result<O::Pointer<O::Dyn>> {
        Ok(O::Pointer::<V>::deserialize(d, ctx)?)
    }
}

pub struct FormatType<O: Object>(PhantomData<O>);

impl<O: Object> Format for FormatType<O> {}

impl<O: Object, V: 'static> VariantFormat<V> for FormatType<O>
where
    O::Pointer<V>: CoerceUnsized<O::Pointer<O::Dyn>> + for<'de> DeserializeJson<'de>,
{
    fn add_object_deserializer(map: &mut TypeMap) {
        map.insert(Box::new(PhantomData::<fn() -> V>) as Box<dyn JsonObjectDeserializer<O>>);
    }
}

pub struct JsonDeserializerTable<O: Object>(Vec<&'static Box<dyn JsonObjectDeserializer<O>>>);

impl<O: Object> JsonDeserializerTable<O> {
    pub fn new() -> Self {
        let object = OBJECT_REGISTRY.object_descriptor::<O>();
        JsonDeserializerTable(
            (0..object.discriminant_names().len())
                .map(|i| object.variant_deserializer(i))
                .collect(),
        )
    }
    pub fn deserialize_object<'p, 'de>(
        &self,
        disc: usize,
        d: <JsonDecoder<'de> as Decoder<'de>>::AnyDecoder<'p>,
        ctx: &mut Context,
    ) -> anyhow::Result<O::Pointer<O::Dyn>> {
        self.0[disc].json_object_deserialize(d, ctx)
    }
}

#[macro_export]
macro_rules! json_format {
    ($carrier:path) => {
        impl<'de> $crate::de::DeserializeVariant<'de, $crate::reexports::marshal_json::decode::full::JsonDecoder<'de>> for $carrier {
            fn deserialize_variant(
                disc: usize,
                d: <$crate::reexports::marshal_json::decode::full::JsonDecoder<'de> as $crate::reexports::marshal::decode::Decoder<'de>>::AnyDecoder<'_>,
                ctx: &mut $crate::reexports::marshal::context::Context,
            ) -> $crate::reexports::anyhow::Result<<$carrier as $crate::Object>::Pointer<<$carrier as $crate::Object>::Dyn>> {
                static DESERIALIZERS: LazyLock<$crate::json_format::JsonDeserializerTable<$carrier>> =
                    LazyLock::new($crate::json_format::JsonDeserializerTable::new);
                DESERIALIZERS.deserialize_object(disc, d, ctx)
            }
        }
    };
}
