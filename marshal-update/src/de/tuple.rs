use marshal::context::Context;
use marshal::de::SchemaError;
use marshal::decode::{AnyGenDecoder, DecodeHint, GenDecoder};

use crate::de::DeserializeUpdate;

impl<D: GenDecoder, T1: DeserializeUpdate<D>, T2: DeserializeUpdate<D>>
    DeserializeUpdate<D> for (T1, T2)
{
    fn deserialize_update<'p, 'de>(
        &mut self,
        d: AnyGenDecoder<'p, 'de, D>,
        mut ctx: Context,
    ) -> anyhow::Result<()> {
        let mut d = d.decode(DecodeHint::Tuple { len: 2 })?.try_into_seq()?;
        self.0
            .deserialize_update(d.decode_next()?.ok_or(SchemaError::TupleTooShort)?, ctx.reborrow())?;
        self.1
            .deserialize_update(d.decode_next()?.ok_or(SchemaError::TupleTooShort)?, ctx.reborrow())?;
        if let Some(_) = d.decode_next()? {
            return Err(SchemaError::TupleTooLong.into());
        }
        Ok(())
    }
}
