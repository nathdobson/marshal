use marshal::context::Context;
use marshal::de::Deserialize;
use marshal::decode::{AnyGenDecoder, GenDecoder};

mod derive_deserialize_update_for_option;
mod option;
mod rc;
mod tuple;

pub trait DeserializeUpdate<D: GenDecoder>: Deserialize<D> {
    fn deserialize_update<'p, 'de>(
        &mut self,
        d: AnyGenDecoder<'p, 'de, D>,
        ctx: Context,
    ) -> anyhow::Result<()>;
}
