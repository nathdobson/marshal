use std::sync;
use std::sync::Arc;

use marshal::context::Context;
use marshal::de::Deserialize;
use marshal::decode::{AnyDecoder, Decoder};

use crate::de::DeserializeUpdate;

impl<'de, D: Decoder<'de>, T: ?Sized> DeserializeUpdate<'de, D> for Arc<T>
where
    Arc<T>: Deserialize<'de, D>,
{
    fn deserialize_update<'p>(
        &mut self,
        d: AnyDecoder<'p, 'de, D>,
         ctx: Context,
    ) -> anyhow::Result<()> {
        if let Some(update) = Option::<Arc<T>>::deserialize(d, ctx)? {
            *self = update;
        }
        Ok(())
    }
}

impl<'de, D: Decoder<'de>, T: ?Sized> DeserializeUpdate<'de, D> for sync::Weak<T>
where
    sync::Weak<T>: Deserialize<'de, D>,
{
    fn deserialize_update<'p>(
        &mut self,
        d: AnyDecoder<'p, 'de, D>,
         ctx: Context,
    ) -> anyhow::Result<()> {
        if let Some(update) = Option::<sync::Weak<T>>::deserialize(d, ctx)? {
            *self = update;
        }
        Ok(())
    }
}
