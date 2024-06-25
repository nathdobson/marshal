use std::any::Any;
use std::sync;
use std::sync::Arc;

use marshal::context::Context;
use marshal_json::decode::full::{JsonDecoder, JsonDecoderBuilder};
use marshal_json::encode::full::{JsonEncoder, JsonEncoderBuilder};
use marshal_shared::de::SharedArcDeserializeContext;
use marshal_shared::ser::SharedSerializeContext;

use crate::de::DeserializeUpdate;
use crate::ser::{DeserializeUpdateDyn, SerializeUpdateDyn};
use crate::tree::de::{DeserializeForest, DynamicDecoder};
use crate::tree::ser::{DynamicEncoder, SerializeForest};
use crate::tree::Tree;

pub trait SerializeUpdateJson: SerializeUpdateDyn<JsonEncoder> {}

impl<T: ?Sized + SerializeUpdateDyn<JsonEncoder>> SerializeUpdateJson for T {}

impl DynamicEncoder for JsonEncoder {
    type SerializeUpdateDyn = dyn SerializeUpdateJson;
}
type JsonSerializeForest = SerializeForest<dyn SerializeUpdateJson>;

pub struct JsonSerializeStream<T> {
    forest: JsonSerializeForest,
    shared: SharedSerializeContext<sync::Weak<Tree<dyn Sync + Send + Any>>>,
    start: Option<Arc<Tree<T>>>,
}

impl<T> JsonSerializeStream<T>
where
    T: Sync + Send + SerializeUpdateJson,
{
    pub fn new(value: Arc<Tree<T>>) -> Self {
        JsonSerializeStream {
            forest: JsonSerializeForest::new(),
            shared: SharedSerializeContext::default(),
            start: Some(value),
        }
    }
    pub fn next(&mut self) -> anyhow::Result<String> {
        let mut ctx = Context::new();
        ctx.insert(&mut self.forest);
        ctx.insert(&mut self.shared);
        if let Some(start) = self.start.take() {
            Ok(JsonEncoderBuilder::new().serialize(&start, &mut ctx)?)
        } else {
            Ok(JsonEncoderBuilder::new()
                .with(|e| JsonSerializeForest::serialize_updates(e, &mut ctx))?)
        }
    }
}

pub trait DeserializeUpdateJson: for<'de> DeserializeUpdateDyn<'de, JsonDecoder<'de>> {}

impl<T: ?Sized + for<'de> DeserializeUpdateDyn<'de, JsonDecoder<'de>>> DeserializeUpdateJson for T {}

pub type JsonDeserializeForest = DeserializeForest<dyn DeserializeUpdateJson>;
pub struct JsonDeserializeStream {
    forest: DeserializeForest<dyn DeserializeUpdateJson>,
    shared: SharedArcDeserializeContext,
}

impl<'de> DynamicDecoder for JsonDecoder<'de> {
    type DeserializeUpdateDyn = dyn DeserializeUpdateJson;
}

impl JsonDeserializeStream {
    pub fn new<T: 'static + Sync + Send + for<'de> DeserializeUpdate<'de, JsonDecoder<'de>>>(
        start: &[u8],
    ) -> anyhow::Result<(Self, Arc<Tree<T>>)> {
        let mut this = JsonDeserializeStream {
            forest: DeserializeForest::<dyn DeserializeUpdateJson>::new(),
            shared: SharedArcDeserializeContext::default(),
        };
        let value = JsonDecoderBuilder::new(start).deserialize(&mut this.ctx())?;
        Ok((this, value))
    }
    fn ctx(&mut self) -> Context {
        let mut ctx = Context::new();
        ctx.insert(&mut self.forest);
        ctx.insert(&mut self.shared);
        ctx
    }
    pub fn next(&mut self, update: &[u8]) -> anyhow::Result<()> {
        JsonDecoderBuilder::new(update)
            .with(|d| JsonDeserializeForest::deserialize_updates(d, &mut self.ctx()))
    }
}
