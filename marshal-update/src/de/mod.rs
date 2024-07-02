use marshal::context::Context;
use marshal::de::Deserialize;
use marshal::decode::{AnyDecoder, Decoder};

mod derive_deserialize_update_for_option;
mod option;
mod rc;
mod tuple;
mod boxed;

pub trait DeserializeUpdate<D: Decoder>: Deserialize<D> {
    fn deserialize_update<'p, 'de>(
        &mut self,
        d: AnyDecoder<'p, 'de, D>,
        ctx: Context,
    ) -> anyhow::Result<()>;
}
