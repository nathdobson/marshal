use marshal_bin::encode::full::BinEncoder;

use crate::ser::SerializeUpdateDyn;
use crate::tree::ser::DynamicEncoder;

pub trait SerializeUpdateBin: Sync + Send + for<'s> SerializeUpdateDyn<BinEncoder<'s>> {}

impl<T: ?Sized + Sync + Send + for<'s> SerializeUpdateDyn<BinEncoder<'s>>> SerializeUpdateBin
    for T
{
}

impl<'s> DynamicEncoder for BinEncoder<'s> {
    type SerializeUpdateDyn = dyn SerializeUpdateBin;
}
