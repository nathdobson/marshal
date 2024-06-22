use std::any::Any;
use std::sync;
use std::sync::Arc;

use crate::de::DeserializeUpdate;
use marshal::context::Context;
use marshal::de::Deserialize;
use marshal::encode::AnyEncoder;
use marshal::reexports::marshal_pointer::arc_ref::ArcRef;
use marshal::ser::rc::SerializeArc;
use marshal::ser::Serialize;
use marshal_json::decode::full::{JsonDecoder, JsonDecoderBuilder};
use marshal_json::encode::full::{JsonEncoder, JsonEncoderBuilder};
use marshal_shared::ser::SharedSerializeContext;

use crate::ser::{DeserializeUpdateDyn, SerializeUpdateDyn};
use crate::tree::de::{DeserializeForest, DynamicDecoder};
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
    T: Sync + Send + SerializeUpdateJson,
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

pub trait DeserializeUpdateJson: for<'de> DeserializeUpdateDyn<'de, JsonDecoder<'de>> {}

impl<T: ?Sized + for<'de> DeserializeUpdateDyn<'de, JsonDecoder<'de>>> DeserializeUpdateJson for T {}

pub type JsonDeserializeForest = DeserializeForest<dyn DeserializeUpdateJson>;
pub struct JsonDeserializeStream {
    ctx: Context,
}

impl<'de> DynamicDecoder for JsonDecoder<'de> {
    type DeserializeUpdateDyn = dyn DeserializeUpdateJson;
}

impl JsonDeserializeStream {
    pub fn new<T: 'static + Sync + Send + for<'de> DeserializeUpdate<'de, JsonDecoder<'de>>>(
        start: &[u8],
    ) -> anyhow::Result<(Self, Arc<Tree<T>>)> {
        let mut ctx = Context::new();
        ctx.insert(DeserializeForest::<dyn DeserializeUpdateJson>::new());
        let value = JsonDecoderBuilder::new(start).deserialize(&mut ctx)?;
        Ok((JsonDeserializeStream { ctx }, value))
    }
    pub fn next(&mut self, update: &[u8]) -> anyhow::Result<()> {
        JsonDecoderBuilder::new(update)
            .with(|d| JsonDeserializeForest::deserialize_updates(d, &mut self.ctx))
    }
}
