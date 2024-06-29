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
use marshal_object::Object;
use marshal_pointer::{AsFlatRef, DowncastRef, RawAny};

use crate::decode::full::BinDecoder;
use crate::encode::full::BinEncoder;
use crate::SerializeBin;

pub trait SerializeDyn = SerializeBin;

pub trait DeserializeVariantBin<O: Object>: 'static + Sync + Send {
    fn bin_deserialize_variant<'p>(
        &self,
        d: AnyDecoder<'p, BinDecoder>,
        ctx: Context,
    ) -> anyhow::Result<O::Pointer<O::Dyn>>;
    fn bin_serialize_variant<'p>(
        &self,
        this: &<O::Pointer<O::Dyn> as AsFlatRef>::FlatRef,
        e: AnyEncoder<'p, BinEncoder>,
        ctx: Context,
    ) -> anyhow::Result<()>;
}

impl<O: Object, V: 'static> DeserializeVariantBin<O> for PhantomData<fn() -> V>
where
    O::Pointer<V>: Deserialize<BinDecoder>,
    O::Pointer<V>: CoerceUnsized<O::Pointer<O::Dyn>>,
    <O::Pointer<O::Dyn> as AsFlatRef>::FlatRef:
        Unsize<<O::Pointer<dyn RawAny> as AsFlatRef>::FlatRef>,
    <O::Pointer<dyn RawAny> as AsFlatRef>::FlatRef:
        DowncastRef<<O::Pointer<V> as AsFlatRef>::FlatRef>,
    <O::Pointer<V> as AsFlatRef>::FlatRef: SerializeBin,
{
    fn bin_deserialize_variant<'p>(
        &self,
        d: AnyDecoder<'p, BinDecoder>,
        mut ctx: Context,
    ) -> anyhow::Result<O::Pointer<O::Dyn>> {
        Ok(<O::Pointer<V>>::deserialize(d, ctx)?)
    }

    fn bin_serialize_variant<'p, 's>(
        &self,
        this: &<O::Pointer<O::Dyn> as AsFlatRef>::FlatRef,
        e: AnyEncoder<'p, BinEncoder>,
        mut ctx: Context,
    ) -> anyhow::Result<()> {
        let upcast = this as &<O::Pointer<dyn RawAny> as AsFlatRef>::FlatRef;
        let downcast = upcast
            .downcast_ref()
            .expect("failed to downcast for serializer");
        <<O::Pointer<V> as AsFlatRef>::FlatRef as Serialize<BinEncoder>>::serialize(
            downcast, e, ctx,
        )
    }
}

pub struct FormatDeserializeProvider<O: Object>(PhantomData<O>);

impl<O: Object> DeserializeProvider for FormatDeserializeProvider<O> {}

impl<O: Object, V: 'static> DeserializeVariantProvider<V> for FormatDeserializeProvider<O>
where
    O::Pointer<V>: CoerceUnsized<O::Pointer<O::Dyn>> + Deserialize<BinDecoder>,
    <O::Pointer<O::Dyn> as AsFlatRef>::FlatRef:
        Unsize<<O::Pointer<dyn RawAny> as AsFlatRef>::FlatRef>,
    <O::Pointer<dyn RawAny> as AsFlatRef>::FlatRef:
        DowncastRef<<O::Pointer<V> as AsFlatRef>::FlatRef>,
    <O::Pointer<V> as AsFlatRef>::FlatRef: SerializeBin,
{
    fn add_deserialize_variant(map: &mut DeserializeVariantSet) {
        map.insert(Box::new(PhantomData::<fn() -> V>) as Box<dyn DeserializeVariantBin<O>>);
    }
}

impl<O: Object> DeserializeVariant for Box<dyn DeserializeVariantBin<O>> {}

#[macro_export]
macro_rules! bin_object {
    ($carrier:path) => {
        const _ : () = {
            static DESERIALIZERS: $crate::reexports::safe_once::sync::LazyLock<$crate::reexports::marshal_object::de::DeserializeVariantTable<$carrier, ::std::boxed::Box<dyn $crate::bin_object::DeserializeVariantBin<$carrier>>>> =
                    $crate::reexports::safe_once::sync::LazyLock::new($crate::reexports::marshal_object::de::DeserializeVariantTable::new);

            impl<'de> $crate::reexports::marshal_object::de::DeserializeVariantForDiscriminant<'de, $crate::decode::full::BinDecoder<'de>> for $carrier {
                fn deserialize_variant<'p>(
                    disc: usize,
                    d: $crate::reexports::marshal::decode::AnyDecoder<'p, 'de, $crate::decode::full::BinDecoder<'de>>,
                    mut ctx: $crate::reexports::marshal::context::Context,
                ) -> $crate::reexports::anyhow::Result<<$carrier as $crate::reexports::marshal_object::Object>::Pointer<<$carrier as $crate::reexports::marshal_object::Object>::Dyn>> {
                    DESERIALIZERS[disc].bin_deserialize_variant(d, ctx)
                }
            }
            impl $crate::reexports::marshal_object::ser::SerializeVariantForDiscriminant<$crate::encode::full::BinEncoder> for $carrier {
                fn serialize_variant(
                    this: &<Self::Pointer<Self::Dyn> as $crate::reexports::marshal_pointer::AsFlatRef>::FlatRef,
                    disc: usize,
                    e: $crate::reexports::marshal::encode::AnyEncoder<'_,$crate::encode::full::BinEncoder>,
                    mut ctx: $crate::reexports::marshal::context::Context
                ) -> $crate::reexports::anyhow::Result<()> {
                    DESERIALIZERS[disc].bin_serialize_variant(this, e, ctx)
                }
            }
        };
    };
}
