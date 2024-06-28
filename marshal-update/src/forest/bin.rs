use crate::forest::de::DynamicDecoder;
use marshal_bin::decode::full::BinDecoder;
use marshal_bin::encode::full::BinEncoder;
use std::any::Any;

use crate::forest::ser::DynamicEncoder;
use crate::ser::{DeserializeUpdateDyn, SerializeUpdateDyn};

pub trait SerializeUpdateBin: Sync + Send + for<'s> SerializeUpdateDyn<BinEncoder> {}

impl<T: ?Sized + Sync + Send + for<'s> SerializeUpdateDyn<BinEncoder>> SerializeUpdateBin
    for T
{
}

impl<'s> DynamicEncoder for BinEncoder {
    type SerializeUpdateDyn = dyn SerializeUpdateBin;
}

pub trait DeserializeUpdateBin:
    Sync + Send + Any + for<'de, 's> DeserializeUpdateDyn<'de, BinDecoder<'de, 's>>
{
}

impl<T: ?Sized + Sync + Send + for<'de, 's> DeserializeUpdateDyn<'de, BinDecoder<'de, 's>>>
    DeserializeUpdateBin for T
{
}

impl<'de, 's> DynamicDecoder for BinDecoder<'de, 's> {
    type DeserializeUpdateDyn = dyn DeserializeUpdateBin;
}
