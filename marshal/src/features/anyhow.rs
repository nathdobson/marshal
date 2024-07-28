use marshal_core::decode::{AnyDecoder, DecodeHint, Decoder};

use crate::context::Context;
use crate::de::Deserialize;
use marshal_core::encode::{AnyEncoder, Encoder};

use crate::ser::Serialize;

impl<D: Decoder> Deserialize<D> for anyhow::Error {
    fn deserialize<'p, 'de>(d: AnyDecoder<'p, 'de, D>, _ctx: Context) -> anyhow::Result<Self> {
        Ok(anyhow::Error::msg(
            d.decode(DecodeHint::String)?
                .try_into_string()?
                .into_owned(),
        ))
    }
}

impl<E: Encoder> Serialize<E> for anyhow::Error {
    fn serialize<'w, 'en>(&self, e: AnyEncoder<'w, 'en, E>, ctx: Context) -> anyhow::Result<()> {
        <String as Serialize<E>>::serialize(&self.to_string(), e, ctx)
    }
}
