use std::any::Any;

use marshal_json::decode::full::JsonGenDecoder;
use marshal_json::encode::full::{JsonEncoder, JsonGenEncoder};

// use crate::forest::de::DynamicDecoder;
use crate::ser::{DeserializeUpdateDyn, SerializeUpdateDyn};

// pub trait SerializeUpdateJson: Any + SerializeUpdateDyn<JsonGenEncoder> {}
//
// impl<T: ?Sized + Any + SerializeUpdateDyn<JsonGenEncoder>> SerializeUpdateJson for T {}
//
// impl DynamicEncoder for JsonGenEncoder {
//     type SerializeUpdateDyn = dyn Sync + Send + SerializeUpdateJson;
// }

// pub trait DeserializeUpdateJson:
//     Sync + Send + Any + DeserializeUpdateDyn<JsonGenDecoder>
// {
// }
//
// impl<T: ?Sized + Sync + Send + DeserializeUpdateDyn<JsonGenDecoder>>
//     DeserializeUpdateJson for T
// {
// }
//
// impl DynamicDecoder for JsonGenDecoder {
//     type DeserializeUpdateDyn = dyn DeserializeUpdateJson;
// }
