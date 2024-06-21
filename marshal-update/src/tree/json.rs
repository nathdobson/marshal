use std::any::Any;
use std::sync;
use std::sync::Arc;

use marshal::context::Context;
use marshal::encode::AnyEncoder;
use marshal::reexports::marshal_pointer::arc_ref::ArcRef;
use marshal::ser::rc::SerializeArc;
use marshal::ser::Serialize;
use marshal_json::encode::full::{JsonEncoder, JsonEncoderBuilder};
use marshal_shared::ser::SharedSerializeContext;

use crate::ser::SerializeUpdateDyn;
use crate::tree::ser::{DynamicEncoder, SerializeForest, SerializeQueue};
use crate::tree::Tree;

pub trait SerializeUpdateJson: SerializeUpdateDyn<JsonEncoder> {}

impl<T: ?Sized + SerializeUpdateDyn<JsonEncoder>> SerializeUpdateJson for T {}

impl DynamicEncoder for JsonEncoder {
    type SerializeUpdateDyn = dyn SerializeUpdateJson;
}
type JsonSerializeForest = SerializeForest<dyn SerializeUpdateJson>;

pub struct JsonSerializeStream<T> {
    ctx: Context,
    start: Option<Arc<Tree<T>>>,
}

impl<T> JsonSerializeStream<T>
where
    T: SerializeUpdateJson,
{
    pub fn new(value: Arc<Tree<T>>) -> Self {
        let mut ctx = Context::new();
        ctx.insert(JsonSerializeForest::new());
        JsonSerializeStream {
            ctx,
            start: Some(value),
        }
    }
    pub fn next(&mut self) -> anyhow::Result<String> {
        if let Some(start) = self.start.take() {
            Ok(JsonEncoderBuilder::new().serialize(&start, &mut self.ctx)?)
        } else {
            Ok(JsonEncoderBuilder::new()
                .with(|e| JsonSerializeForest::serialize_updates(e, &mut self.ctx))?)
        }
    }
}
