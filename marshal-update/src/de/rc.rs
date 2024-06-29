use std::sync;
use std::sync::Arc;

use marshal::context::Context;
use marshal::de::Deserialize;
use marshal::decode::{AnyDecoder, Decoder};

use crate::de::DeserializeUpdate;

impl<D: Decoder, T: ?Sized> DeserializeUpdate<D> for Arc<T>
where
    Arc<T>: Deserialize<D>,
{
    fn deserialize_update<'p, 'de>(
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

impl<D: Decoder, T: ?Sized> DeserializeUpdate<D> for sync::Weak<T>
where
    sync::Weak<T>: Deserialize<D>,
{
    fn deserialize_update<'p, 'de>(
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
