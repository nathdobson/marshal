use marshal_core::decode::{AnyDecoder, DecodeHint, Decoder};

use crate::context::Context;
use crate::de::Deserialize;

impl<D:Decoder> Deserialize<D> for anyhow::Error{
    fn deserialize<'p, 'de>(d: AnyDecoder<'p, 'de, D>, ctx: Context) -> anyhow::Result<Self>
    {
        Ok(anyhow::Error::msg(d.decode(DecodeHint::String)?.try_into_string()?.into_owned()))
    }
}