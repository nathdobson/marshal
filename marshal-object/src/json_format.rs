use std::any::{type_name, TypeId};
use std::marker::PhantomData;
use std::ops::CoerceUnsized;

use type_map::concurrent::TypeMap;

use crate::de::{Format, VariantFormat};
use marshal::context::Context;
use marshal::de::Deserialize;
use marshal::decode::Decoder;
use marshal::ser::Serialize;
use marshal_json::decode::full::JsonDecoder;
use marshal_json::encode::full::JsonEncoder;

use crate::{ObjectPointer, VariantPointer, OBJECT_REGISTRY};

pub trait SerializeDyn = Serialize<JsonEncoder>;

pub trait JsonObjectDeserializer<OP: ObjectPointer>: 'static + Sync + Send {
    fn json_object_deserialize<'p, 'de>(
        &self,
        d: <JsonDecoder<'de> as Decoder<'de>>::AnyDecoder<'p>,
        ctx: &mut Context,
    ) -> anyhow::Result<OP>;
}

impl<
        OP: ObjectPointer,
        VP: VariantPointer + CoerceUnsized<OP> + for<'de> Deserialize<'de, JsonDecoder<'de>>,
    > JsonObjectDeserializer<OP> for PhantomData<fn() -> VP>
{
    fn json_object_deserialize<'p, 'de, 's>(
        &self,
        d: <JsonDecoder<'de> as Decoder<'de>>::AnyDecoder<'p>,
        ctx: &mut Context,
    ) -> anyhow::Result<OP> {
        Ok(VP::deserialize(d, ctx)?)
    }
}

pub struct FormatType;

impl Format for FormatType {}

impl<VP: VariantPointer + for<'de> Deserialize<'de, JsonDecoder<'de>>> VariantFormat<VP>
    for FormatType
{
    fn add_object_deserializer<OP: ObjectPointer>(map: &mut TypeMap)
    where
        VP: CoerceUnsized<OP>,
    {
        map.insert(Box::new(PhantomData::<fn() -> VP>) as Box<dyn JsonObjectDeserializer<OP>>);
    }
}

pub struct JsonDeserializerTable<OP: ObjectPointer>(
    Vec<&'static Box<dyn JsonObjectDeserializer<OP>>>,
);

impl<OP: ObjectPointer> JsonDeserializerTable<OP> {
    pub fn new() -> Self {
        let object = OBJECT_REGISTRY
            .object_descriptor(TypeId::of::<OP::Object>())
            .unwrap_or_else(|| {
                panic!(
                    "could not find object descriptor for {}",
                    type_name::<OP::Object>()
                )
            });
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
    ) -> anyhow::Result<OP> {
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
                    LazyLock::new($crate::json_format::JsonDeserializerTable::new);
                DESERIALIZERS.deserialize_object(disc, d, ctx)
            }
        }
    };
}
