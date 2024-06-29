// pub trait SerializeUpdateJson: Any + SerializeUpdateDyn<JsonEncoder> {}
//
// impl<T: ?Sized + Any + SerializeUpdateDyn<JsonEncoder>> SerializeUpdateJson for T {}
//
// impl DynamicEncoder for JsonEncoder {
//     type SerializeUpdateDyn = dyn Sync + Send + SerializeUpdateJson;
// }

// pub trait DeserializeUpdateJson:
//     Sync + Send + Any + DeserializeUpdateDyn<JsonDecoder>
// {
// }
//
// impl<T: ?Sized + Sync + Send + DeserializeUpdateDyn<JsonDecoder>>
//     DeserializeUpdateJson for T
// {
// }
//
// impl DynamicDecoder for JsonDecoder {
//     type DeserializeUpdateDyn = dyn DeserializeUpdateJson;
// }
