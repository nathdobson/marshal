use marshal::context::Context;
use marshal::de::Deserialize;
use marshal::decode::{AnyDecoder, Decoder};
use marshal::encode::{AnyEncoder, Encoder};
use marshal::ser::Serialize;
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};

static FOREST_ID: AtomicU64 = AtomicU64::new(0);

#[derive(Copy, Clone, Eq, Ord, PartialOrd, PartialEq, Hash, Debug)]
pub struct ForestId(u64);

impl ForestId {
    pub fn new() -> Self {
        ForestId(FOREST_ID.fetch_add(1, Ordering::Relaxed))
    }
}

impl<E: Encoder> Serialize<E> for ForestId {
    fn serialize(&self, e: AnyEncoder<'_, E>, ctx: Context) -> anyhow::Result<()> {
        todo!()
    }
}

impl<'de, D: Decoder<'de>> Deserialize<'de, D> for ForestId {
    fn deserialize<'p>(d: AnyDecoder<'p, 'de, D>, ctx: Context) -> anyhow::Result<Self> {
        todo!()
    }
}

pub struct ForestDeserializeContext {
    renaming: HashMap<u64, ForestId>,
}
