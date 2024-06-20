mod derive_deserialize_update_for_option;
mod rc;

use marshal::context::Context;
use marshal::de::Deserialize;
use marshal::decode::{AnyDecoder, Decoder};

pub trait DeserializeUpdate<'de, D: Decoder<'de>>: Deserialize<'de, D> {
    fn deserialize_update<'p>(
        &mut self,
        d: AnyDecoder<'p, 'de, D>,
        ctx: &mut Context,
    ) -> anyhow::Result<()>;
}
