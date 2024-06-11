use std::any::TypeId;
use std::marker::{PhantomData, Unsize};

use type_map::concurrent::TypeMap;

use crate::de::{Format, VariantFormat};
use crate::OBJECT_REGISTRY;
use marshal::context::Context;
use marshal::de::Deserialize;
use marshal::decode::Decoder;
use marshal::ser::Serialize;
use marshal_bin::decode::full::BinDecoder;
use marshal_bin::encode::full::BinEncoder;

pub trait SerializeDyn = for<'s> Serialize<BinEncoder<'s>>;

pub trait BinObjectDeserializer<T: ?Sized>: 'static + Sync + Send {
    fn bin_object_deserialize<'p, 'de, 's>(
        &self,
        d: <BinDecoder<'de, 's> as Decoder<'de>>::AnyDecoder<'p>,
        ctx: &mut Context,
    ) -> anyhow::Result<Box<T>>;
}

impl<
        T: 'static + ?Sized,
        V: 'static + Unsize<T> + for<'de, 's> Deserialize<'de, BinDecoder<'de, 's>>,
    > BinObjectDeserializer<T> for PhantomData<fn() -> V>
{
    fn bin_object_deserialize<'p, 'de, 's>(
        &self,
        d: <BinDecoder<'de, 's> as Decoder<'de>>::AnyDecoder<'p>,
        ctx: &mut Context,
    ) -> anyhow::Result<Box<T>> {
        Ok(Box::<V>::new(V::deserialize(d, ctx)?))
    }
}

pub struct FormatType;

impl Format for FormatType {}

impl<V: 'static + for<'de, 's> Deserialize<'de, BinDecoder<'de, 's>>> VariantFormat<V>
    for FormatType
{
    fn add_object_deserializer<T: 'static + ?Sized>(map: &mut TypeMap)
    where
        V: Unsize<T>,
    {
        map.insert(Box::new(PhantomData::<fn() -> V>) as Box<dyn BinObjectDeserializer<T>>);
    }
}

pub struct BinDeserializerTable<T: 'static + ?Sized>(
    Vec<&'static Box<dyn BinObjectDeserializer<T>>>,
);

impl<T: 'static + ?Sized> BinDeserializerTable<T> {
    pub fn new() -> Self {
        let object = OBJECT_REGISTRY
            .object_descriptor(TypeId::of::<T>())
            .unwrap();
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
    ) -> anyhow::Result<Box<T>> {
        self.0[disc].bin_object_deserialize(d, ctx)
    }
}

#[macro_export]
macro_rules! bin_format {
    ($tr:ident) => {
        impl<'de,'s> $crate::de::DeserializeVariant<'de, $crate::reexports::marshal_bin::decode::full::BinDecoder<'de,'s>> for dyn $tr {
            fn deserialize_variant(
                disc: usize,
                d: <$crate::reexports::marshal_bin::decode::full::BinDecoder<'de,'s> as $crate::reexports::marshal::decode::Decoder<'de>>::AnyDecoder<'_>,
                ctx: &mut $crate::reexports::marshal::context::Context,
            ) -> $crate::reexports::anyhow::Result<::std::boxed::Box<Self>> {
                static DESERIALIZERS: LazyLock<$crate::bin_format::BinDeserializerTable<dyn $tr>> =
                    LazyLock::new($crate::bin_format::BinDeserializerTable::new);
                DESERIALIZERS.deserialize_object(disc, d, ctx)
            }
        }
    };
}
