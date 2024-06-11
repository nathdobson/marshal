use std::marker::{PhantomData};
use std::ops::CoerceUnsized;

use type_map::concurrent::TypeMap;

use crate::de::{Format, VariantFormat};
use crate::{Object, OBJECT_REGISTRY};
use marshal::context::Context;
use marshal::de::Deserialize;
use marshal::decode::Decoder;
use marshal_bin::decode::full::BinDecoder;
use marshal_bin::SerializeBin;

pub trait SerializeDyn = SerializeBin;

pub trait BinObjectDeserializer<O: Object>: 'static + Sync + Send {
    fn bin_object_deserialize<'p, 'de, 's>(
        &self,
        d: <BinDecoder<'de, 's> as Decoder<'de>>::AnyDecoder<'p>,
        ctx: &mut Context,
    ) -> anyhow::Result<O::Pointer<O::Dyn>>;
}

impl<O: Object, V: 'static> BinObjectDeserializer<O> for PhantomData<fn() -> V>
where
    O::Pointer<V>: for<'de, 's> Deserialize<'de, BinDecoder<'de, 's>>,
    O::Pointer<V>: CoerceUnsized<O::Pointer<O::Dyn>>,
{
    fn bin_object_deserialize<'p, 'de, 's>(
        &self,
        d: <BinDecoder<'de, 's> as Decoder<'de>>::AnyDecoder<'p>,
        ctx: &mut Context,
    ) -> anyhow::Result<O::Pointer<O::Dyn>> {
        Ok(<O::Pointer<V>>::deserialize(d, ctx)?)
    }
}

pub struct FormatType<O: Object>(PhantomData<O>);

impl<O: Object> Format for FormatType<O> {}

impl<O: Object, V: 'static> VariantFormat<V> for FormatType<O>
where
    O::Pointer<V>:
        CoerceUnsized<O::Pointer<O::Dyn>> + for<'de, 's> Deserialize<'de, BinDecoder<'de, 's>>,
{
    fn add_object_deserializer(map: &mut TypeMap) {
        map.insert(Box::new(PhantomData::<fn() -> V>) as Box<dyn BinObjectDeserializer<O>>);
    }
}

pub struct BinDeserializerTable<O: Object>(Vec<&'static Box<dyn BinObjectDeserializer<O>>>);

impl<O: Object> BinDeserializerTable<O> {
    pub fn new() -> Self {
        let object = OBJECT_REGISTRY.object_descriptor::<O>();
        BinDeserializerTable(
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
    ) -> anyhow::Result<O::Pointer<O::Dyn>> {
        self.0[disc].bin_object_deserialize(d, ctx)
    }
}

#[macro_export]
macro_rules! bin_format {
    ($carrier:path) => {
        impl<'de,'s> $crate::de::DeserializeVariant<'de, $crate::reexports::marshal_bin::decode::full::BinDecoder<'de,'s>> for $carrier {
            fn deserialize_variant(
                disc: usize,
                d: <$crate::reexports::marshal_bin::decode::full::BinDecoder<'de,'s> as $crate::reexports::marshal::decode::Decoder<'de>>::AnyDecoder<'_>,
                ctx: &mut $crate::reexports::marshal::context::Context,
            ) -> $crate::reexports::anyhow::Result<<$carrier as $crate::Object>::Pointer<<$carrier as $crate::Object>::Dyn>> {
                static DESERIALIZERS: LazyLock<$crate::bin_format::BinDeserializerTable<$carrier>> =
                    LazyLock::new($crate::bin_format::BinDeserializerTable::new);
                DESERIALIZERS.deserialize_object(disc, d, ctx)
            }
        }
    };
}
