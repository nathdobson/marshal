use marshal::context::Context;
use marshal::de::SchemaError;
use marshal::decode::{AnyDecoder, DecodeHint, Decoder};

use crate::de::DeserializeUpdate;

impl<'de, D: Decoder<'de>, T1: DeserializeUpdate<'de, D>, T2: DeserializeUpdate<'de, D>>
    DeserializeUpdate<'de, D> for (T1, T2)
{
    fn deserialize_update<'p>(
        &mut self,
        d: AnyDecoder<'p, 'de, D>,
        ctx: &mut Context,
    ) -> anyhow::Result<()> {
        let mut d = d.decode(DecodeHint::Tuple { len: 2 })?.try_into_seq()?;
        self.0
            .deserialize_update(d.decode_next()?.ok_or(SchemaError::TupleTooShort)?, ctx)?;
        self.1
            .deserialize_update(d.decode_next()?.ok_or(SchemaError::TupleTooShort)?, ctx)?;
        if let Some(_) = d.decode_next()? {
            return Err(SchemaError::TupleTooLong.into());
        }
        Ok(())
    }
}
