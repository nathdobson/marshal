use std::any::Any;

use marshal_json::decode::full::JsonDecoder;
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
    Sync + Send + Any + for<'de> DeserializeUpdateDyn<'de, JsonDecoder<'de>>
{
}

impl<T: ?Sized + Sync + Send + for<'de> DeserializeUpdateDyn<'de, JsonDecoder<'de>>>
    DeserializeUpdateJson for T
{
}

impl<'de> DynamicDecoder for JsonDecoder<'de> {
    type DeserializeUpdateDyn = dyn DeserializeUpdateJson;
}
