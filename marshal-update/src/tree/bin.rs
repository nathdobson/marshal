use std::any::Any;
use std::sync;
use std::sync::Arc;

use marshal::context::Context;
use marshal::encode::AnyEncoder;
use marshal::reexports::marshal_pointer::arc_ref::ArcRef;
use marshal::ser::rc::SerializeArc;
use marshal::ser::Serialize;
use marshal_bin::encode::full::BinEncoder;
use marshal_json::encode::full::JsonEncoder;
use marshal_shared::ser::SharedSerializeContext;

use crate::ser::SerializeUpdateDyn;
use crate::tree::json::SerializeUpdateJson;
use crate::tree::ser::{DynamicEncoder, SerializeForest};
use crate::tree::Tree;

pub trait SerializeUpdateBin: for<'s> SerializeUpdateDyn<BinEncoder<'s>> {}

impl<T: ?Sized + for<'s> SerializeUpdateDyn<BinEncoder<'s>>> SerializeUpdateBin for T {}

impl<'s> DynamicEncoder for BinEncoder<'s> {
    type SerializeUpdateDyn = dyn SerializeUpdateBin;
}
