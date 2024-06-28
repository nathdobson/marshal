use crate::forest::de::DynamicDecoder;
use marshal_bin::decode::full::BinDecoder;
use marshal_bin::encode::full::BinEncoder;
use std::any::Any;

use crate::forest::ser::DynamicEncoder;
use crate::ser::{DeserializeUpdateDyn, SerializeUpdateDyn};

pub trait SerializeUpdateBin: Sync + Send + for<'s> SerializeUpdateDyn<BinEncoder> {}

impl<T: ?Sized + Sync + Send + for<'s> SerializeUpdateDyn<BinEncoder>> SerializeUpdateBin for T {}

impl<'s> DynamicEncoder for BinEncoder {
    type SerializeUpdateDyn = dyn SerializeUpdateBin;
}

pub trait DeserializeUpdateBin:
    Sync + Send + Any + for<'de> DeserializeUpdateDyn<'de, BinDecoder<'de>>
{
}

impl<T: ?Sized + Sync + Send + for<'de> DeserializeUpdateDyn<'de, BinDecoder<'de>>>
    DeserializeUpdateBin for T
{
}

impl<'de> DynamicDecoder for BinDecoder<'de> {
    type DeserializeUpdateDyn = dyn DeserializeUpdateBin;
}
