use marshal::context::Context;
use marshal::de::Deserialize;
use marshal::decode::{AnyDecoder, Decoder};
use crate::de::DeserializeUpdate;

impl<D: Decoder, T: ?Sized + DeserializeUpdate<D>> DeserializeUpdate<D> for Box<T>
where
    Box<T>: Deserialize<D>,
{
    fn deserialize_update<'p, 'de>(
        &mut self,
        d: AnyDecoder<'p, 'de, D>,
        ctx: Context,
    ) -> anyhow::Result<()> {
        (**self).deserialize_update(d, ctx)
    }
}
