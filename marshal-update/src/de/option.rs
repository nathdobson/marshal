use marshal::context::Context;
use marshal::de::Deserialize;
use marshal::decode::{AnyDecoder, DecodeHint, Decoder};

use crate::de::DeserializeUpdate;

impl<D: Decoder, T: DeserializeUpdate<D>> DeserializeUpdate<D> for Option<T> {
    fn deserialize_update<'p, 'de>(
        &mut self,
        d: AnyDecoder<'p, 'de, D>,
        ctx: Context,
    ) -> anyhow::Result<()> {
        if let Some(this) = self {
            if let Some(mut d) = d.decode(DecodeHint::Option)?.try_into_option()? {
                this.deserialize_update(d.decode_some()?, ctx)?;
                d.decode_end()?;
            } else {
                *self = None;
            }
        } else {
            *self = Self::deserialize(d, ctx)?;
        }
        Ok(())
    }
}
