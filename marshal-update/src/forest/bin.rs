// pub trait SerializeUpdateBin: Sync + Send + for<'s> SerializeUpdateDyn<BinEncoder<'s>> {}
//
// impl<T: ?Sized + Sync + Send + for<'s> SerializeUpdateDyn<BinEncoder<'s>>> SerializeUpdateBin
//     for T
// {
// }
//
// impl<'s> DynamicEncoder for BinEncoder<'s> {
//     type SerializeUpdateDyn = dyn SerializeUpdateBin;
// }

// pub trait DeserializeUpdateBin: Sync + Send + Any + DeserializeUpdateDyn<BinGenDecoder> {}
//
// impl<T: ?Sized + Sync + Send + DeserializeUpdateDyn<BinGenDecoder>> DeserializeUpdateBin for T {}

// impl DynamicDecoder for BinGenDecoder {
//     type DeserializeUpdateDyn = dyn DeserializeUpdateBin;
// }
