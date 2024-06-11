use std::any::{type_name, TypeId};
use std::marker::{PhantomData};
use std::ops::{CoerceUnsized, Deref};

use type_map::concurrent::TypeMap;

use crate::de::{Format, VariantFormat};
use marshal::context::Context;
use marshal::de::Deserialize;
use marshal::decode::Decoder;
use marshal::ser::Serialize;
use marshal_json::decode::full::JsonDecoder;
use marshal_json::encode::full::JsonEncoder;

use crate::OBJECT_REGISTRY;

pub trait SerializeDyn = Serialize<JsonEncoder>;

pub trait JsonObjectDeserializer<T: Deref>: 'static + Sync + Send {
    fn json_object_deserialize<'p, 'de>(
        &self,
        d: <JsonDecoder<'de> as Decoder<'de>>::AnyDecoder<'p>,
        ctx: &mut Context,
    ) -> anyhow::Result<T>;
}

impl<
        T: 'static + Deref,
        V: 'static + CoerceUnsized<T> + for<'de> Deserialize<'de, JsonDecoder<'de>>,
    > JsonObjectDeserializer<T> for PhantomData<fn() -> V>
{
    fn json_object_deserialize<'p, 'de, 's>(
        &self,
        d: <JsonDecoder<'de> as Decoder<'de>>::AnyDecoder<'p>,
        ctx: &mut Context,
    ) -> anyhow::Result<T> {
        Ok(V::deserialize(d, ctx)?)
    }
}

pub struct FormatType;

impl Format for FormatType {}

impl<V: 'static + Deref + for<'de> Deserialize<'de, JsonDecoder<'de>>> VariantFormat<V>
    for FormatType
{
    fn add_object_deserializer<T: 'static + Deref>(map: &mut TypeMap)
    where
        V: CoerceUnsized<T>,
    {
        map.insert(Box::new(PhantomData::<fn() -> V>) as Box<dyn JsonObjectDeserializer<T>>);
    }
}

pub struct JsonDeserializerTable<T: 'static + ?Sized>(
    Vec<&'static Box<dyn JsonObjectDeserializer<T>>>,
);

impl<T: 'static + Deref> JsonDeserializerTable<T> {
    pub fn new<O:'static+?Sized>() -> Self {
        let object = OBJECT_REGISTRY
            .object_descriptor(TypeId::of::<O>())
            .unwrap_or_else(|| panic!("could not find object descriptor for {}", type_name::<T>()));
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
    ) -> anyhow::Result<T> {
        self.0[disc].json_object_deserialize(d, ctx)
    }
}

#[macro_export]
macro_rules! json_format {
    ($tr:ident) => {
        impl<'de> $crate::de::DeserializeVariant<'de, $crate::reexports::marshal_json::decode::full::JsonDecoder<'de>, std::boxed::Box<dyn $tr>> for dyn $tr {
            fn deserialize_variant(
                disc: usize,
                d: <$crate::reexports::marshal_json::decode::full::JsonDecoder<'de> as $crate::reexports::marshal::decode::Decoder<'de>>::AnyDecoder<'_>,
                ctx: &mut $crate::reexports::marshal::context::Context,
            ) -> $crate::reexports::anyhow::Result<::std::boxed::Box<Self>> {
                static DESERIALIZERS: LazyLock<$crate::json_format::JsonDeserializerTable<Box<dyn $tr>>> =
                    LazyLock::new($crate::json_format::JsonDeserializerTable::new::<dyn $tr>);
                DESERIALIZERS.deserialize_object(disc, d, ctx)
            }
        }
    };
}
