use std::any::Any;

use marshal_json::decode::full::JsonGenDecoder;
use marshal_json::encode::full::JsonEncoder;

use crate::forest::de::DynamicDecoder;
use crate::forest::ser::DynamicEncoder;
use crate::ser::{DeserializeUpdateDyn, SerializeUpdateDyn};

pub trait SerializeUpdateJson: Any + SerializeUpdateDyn<JsonEncoder> {}

impl<T: ?Sized + Any + SerializeUpdateDyn<JsonEncoder>> SerializeUpdateJson for T {}

impl DynamicEncoder for JsonEncoder {
    type SerializeUpdateDyn = dyn Sync + Send + SerializeUpdateJson;
}

pub trait DeserializeUpdateJson:
    Sync + Send + Any + DeserializeUpdateDyn<JsonGenDecoder>
{
}

impl<T: ?Sized + Sync + Send + DeserializeUpdateDyn<JsonGenDecoder>>
    DeserializeUpdateJson for T
{
}

impl DynamicDecoder for JsonGenDecoder {
    type DeserializeUpdateDyn = dyn DeserializeUpdateJson;
}
