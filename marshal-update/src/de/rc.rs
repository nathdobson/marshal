use marshal::context::Context;
use marshal::de::Deserialize;
use marshal::decode::{AnyDecoder, Decoder};
use marshal_pointer::{Arcf, ArcfWeak};

use crate::de::DeserializeUpdate;

impl<D: Decoder, T: ?Sized> DeserializeUpdate<D> for Arcf<T>
where
    Arcf<T>: Deserialize<D>,
{
    fn deserialize_update<'p, 'de>(
        &mut self,
        d: AnyDecoder<'p, 'de, D>,
        ctx: Context,
    ) -> anyhow::Result<()> {
        if let Some(update) = Option::<Arcf<T>>::deserialize(d, ctx)? {
            *self = update;
        }
        Ok(())
    }
}

impl<D: Decoder, T: ?Sized> DeserializeUpdate<D> for ArcfWeak<T>
where
    ArcfWeak<T>: Deserialize<D>,
{
    fn deserialize_update<'p, 'de>(
        &mut self,
        d: AnyDecoder<'p, 'de, D>,
        ctx: Context,
    ) -> anyhow::Result<()> {
        if let Some(update) = Option::<ArcfWeak<T>>::deserialize(d, ctx)? {
            *self = update;
        }
        Ok(())
    }
}
