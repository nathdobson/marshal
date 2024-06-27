use std::sync::atomic::{AtomicU64, Ordering};

use marshal::context::Context;
use marshal::de::Deserialize;
use marshal::decode::{AnyDecoder, Decoder};
use marshal::encode::{AnyEncoder, Encoder};
use marshal::ser::Serialize;

static FOREST_ID: AtomicU64 = AtomicU64::new(0);

#[derive(Copy, Clone, Eq, Ord, PartialOrd, PartialEq, Hash, Debug)]
pub struct ForestId(u64);

impl ForestId {
    pub fn new() -> Self {
        ForestId(FOREST_ID.fetch_add(1, Ordering::Relaxed))
    }
}

impl<E: Encoder> Serialize<E> for ForestId {
    fn serialize(&self, _e: AnyEncoder<'_, E>, _ctx: Context) -> anyhow::Result<()> {
        todo!()
    }
}

impl<'de, D: Decoder<'de>> Deserialize<'de, D> for ForestId {
    fn deserialize<'p>(_d: AnyDecoder<'p, 'de, D>, _ctx: Context) -> anyhow::Result<Self> {
        todo!()
    }
}

