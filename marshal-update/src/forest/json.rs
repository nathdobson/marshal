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
