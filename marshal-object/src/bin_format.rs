use std::marker::PhantomData;
use std::ops::CoerceUnsized;

use type_map::concurrent::TypeMap;

use crate::de::{DeserializeProvider, DeserializeVariant, DeserializeVariantProvider};
use crate::{Object};
use marshal::context::Context;
use marshal::de::Deserialize;
use marshal::decode::Decoder;
use marshal_bin::decode::full::BinDecoder;
use marshal_bin::SerializeBin;

pub trait SerializeDyn = SerializeBin;

pub trait DeserializeVariantBin<O: Object>: 'static + Sync + Send {
    fn bin_deserialize_variant<'p, 'de, 's>(
        &self,
        d: <BinDecoder<'de, 's> as Decoder<'de>>::AnyDecoder<'p>,
        ctx: &mut Context,
    ) -> anyhow::Result<O::Pointer<O::Dyn>>;
}

impl<O: Object, V: 'static> DeserializeVariantBin<O> for PhantomData<fn() -> V>
where
    O::Pointer<V>: for<'de, 's> Deserialize<'de, BinDecoder<'de, 's>>,
    O::Pointer<V>: CoerceUnsized<O::Pointer<O::Dyn>>,
{
    fn bin_deserialize_variant<'p, 'de, 's>(
        &self,
        d: <BinDecoder<'de, 's> as Decoder<'de>>::AnyDecoder<'p>,
        ctx: &mut Context,
    ) -> anyhow::Result<O::Pointer<O::Dyn>> {
        Ok(<O::Pointer<V>>::deserialize(d, ctx)?)
    }
}

pub struct FormatDeserializeProvider<O: Object>(PhantomData<O>);

impl<O: Object> DeserializeProvider for FormatDeserializeProvider<O> {}

impl<O: Object, V: 'static> DeserializeVariantProvider<V> for FormatDeserializeProvider<O>
where
    O::Pointer<V>:
        CoerceUnsized<O::Pointer<O::Dyn>> + for<'de, 's> Deserialize<'de, BinDecoder<'de, 's>>,
{
    fn add_deserialize_variant(map: &mut TypeMap) {
        map.insert(Box::new(PhantomData::<fn() -> V>) as Box<dyn DeserializeVariantBin<O>>);
    }
}

impl<O: Object> DeserializeVariant for Box<dyn DeserializeVariantBin<O>> {}

#[macro_export]
macro_rules! bin_format {
    ($carrier:path) => {
        impl<'de,'s> $crate::de::DeserializeVariantForDiscriminant<'de, $crate::reexports::marshal_bin::decode::full::BinDecoder<'de,'s>> for $carrier {
            fn deserialize_variant(
                disc: usize,
                d: <$crate::reexports::marshal_bin::decode::full::BinDecoder<'de,'s> as $crate::reexports::marshal::decode::Decoder<'de>>::AnyDecoder<'_>,
                ctx: &mut $crate::reexports::marshal::context::Context,
            ) -> $crate::reexports::anyhow::Result<<$carrier as $crate::Object>::Pointer<<$carrier as $crate::Object>::Dyn>> {
                static DESERIALIZERS: LazyLock<$crate::de::DeserializeVariantTable<$carrier, ::std::boxed::Box<dyn $crate::bin_format::DeserializeVariantBin<$carrier>>>> =
                    LazyLock::new($crate::de::DeserializeVariantTable::new);
                DESERIALIZERS[disc].bin_deserialize_variant(d, ctx)
            }
        }
    };
}
