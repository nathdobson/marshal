use crate::de::DeserializeUpdate;
use marshal::context::Context;
use marshal::de::Deserialize;
use marshal::decode::{AnyDecoder, Decoder};
use std::sync::Arc;

impl<'de, D: Decoder<'de>, T: ?Sized> DeserializeUpdate<'de, D> for Arc<T>
where
    Arc<T>: Deserialize<'de, D>,
{
    fn deserialize_update<'p>(
        &mut self,
        d: AnyDecoder<'p, 'de, D>,
        ctx: &mut Context,
    ) -> anyhow::Result<()> {
        if let Some(update) = Option::<Arc<T>>::deserialize(d, ctx)? {
            *self = update;
        }
        Ok(())
    }
}
