use std::any::Any;

use marshal_bin::decode::full::BinGenDecoder;
use marshal_bin::encode::full::BinEncoder;

use crate::forest::de::DynamicDecoder;
use crate::forest::ser::DynamicEncoder;
use crate::ser::{DeserializeUpdateDyn, SerializeUpdateDyn};

pub trait SerializeUpdateBin: Sync + Send + for<'s> SerializeUpdateDyn<BinEncoder<'s>> {}

impl<T: ?Sized + Sync + Send + for<'s> SerializeUpdateDyn<BinEncoder<'s>>> SerializeUpdateBin
    for T
{
}

impl<'s> DynamicEncoder for BinEncoder<'s> {
    type SerializeUpdateDyn = dyn SerializeUpdateBin;
}

pub trait DeserializeUpdateBin: Sync + Send + Any + DeserializeUpdateDyn<BinGenDecoder> {}

impl<T: ?Sized + Sync + Send + DeserializeUpdateDyn<BinGenDecoder>> DeserializeUpdateBin for T {}

impl DynamicDecoder for BinGenDecoder {
    type DeserializeUpdateDyn = dyn DeserializeUpdateBin;
}
