use std::any::{type_name, TypeId};
use std::marker::PhantomData;
use std::ops::CoerceUnsized;

use type_map::concurrent::TypeMap;

use crate::de::{Format, VariantFormat};
use crate::{ObjectPointer, VariantPointer, OBJECT_REGISTRY};
use marshal::context::Context;
use marshal::de::Deserialize;
use marshal::decode::Decoder;
use marshal::ser::Serialize;
use marshal_bin::decode::full::BinDecoder;
use marshal_bin::encode::full::BinEncoder;

pub trait SerializeDyn = for<'s> Serialize<BinEncoder<'s>>;

pub trait BinObjectDeserializer<OP: ObjectPointer>: 'static + Sync + Send {
    fn bin_object_deserialize<'p, 'de, 's>(
        &self,
        d: <BinDecoder<'de, 's> as Decoder<'de>>::AnyDecoder<'p>,
        ctx: &mut Context,
    ) -> anyhow::Result<OP>;
}

impl<
        OP: ObjectPointer,
        VP: VariantPointer + CoerceUnsized<OP> + for<'de, 's> Deserialize<'de, BinDecoder<'de, 's>>,
    > BinObjectDeserializer<OP> for PhantomData<fn() -> VP>
{
    fn bin_object_deserialize<'p, 'de, 's>(
        &self,
        d: <BinDecoder<'de, 's> as Decoder<'de>>::AnyDecoder<'p>,
        ctx: &mut Context,
    ) -> anyhow::Result<OP> {
        Ok(VP::deserialize(d, ctx)?)
    }
}

pub struct FormatType;

impl Format for FormatType {}

impl<VP: VariantPointer + for<'de, 's> Deserialize<'de, BinDecoder<'de, 's>>> VariantFormat<VP>
    for FormatType
{
    fn add_object_deserializer<OP: ObjectPointer>(map: &mut TypeMap)
    where
        VP: CoerceUnsized<OP>,
    {
        map.insert(Box::new(PhantomData::<fn() -> VP>) as Box<dyn BinObjectDeserializer<OP>>);
    }
}

pub struct BinDeserializerTable<OP: ObjectPointer>(
    Vec<&'static Box<dyn BinObjectDeserializer<OP>>>,
);

impl<OP: ObjectPointer> BinDeserializerTable<OP> {
    pub fn new() -> Self {
        let object = OBJECT_REGISTRY
            .object_descriptor(TypeId::of::<OP::Object>())
            .unwrap_or_else(|| panic!("could not find object descriptor for {}", type_name::<OP::Object>()));
        crate::bin_format::BinDeserializerTable(
            (0..object.discriminant_names().len())
                .map(|i| object.variant_deserializer(i))
                .collect(),
        )
    }
    pub fn deserialize_object<'p, 'de, 's>(
        &self,
        disc: usize,
        d: <BinDecoder<'de, 's> as Decoder<'de>>::AnyDecoder<'p>,
        ctx: &mut Context,
    ) -> anyhow::Result<OP> {
        self.0[disc].bin_object_deserialize(d, ctx)
    }
}

#[macro_export]
macro_rules! bin_format {
    ($tr:ident) => {
        impl<'de,'s> $crate::de::DeserializeVariant<'de, $crate::reexports::marshal_bin::decode::full::BinDecoder<'de,'s>, ::std::boxed::Box<dyn $tr>> for dyn $tr {
            fn deserialize_variant(
                disc: usize,
                d: <$crate::reexports::marshal_bin::decode::full::BinDecoder<'de,'s> as $crate::reexports::marshal::decode::Decoder<'de>>::AnyDecoder<'_>,
                ctx: &mut $crate::reexports::marshal::context::Context,
            ) -> $crate::reexports::anyhow::Result<::std::boxed::Box<Self>> {
                static DESERIALIZERS: LazyLock<$crate::bin_format::BinDeserializerTable<Box<dyn $tr>>> =
                    LazyLock::new($crate::bin_format::BinDeserializerTable::new);
                DESERIALIZERS.deserialize_object(disc, d, ctx)
            }
        }
    };
}
