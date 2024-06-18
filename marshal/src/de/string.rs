use marshal_core::decode::{AnyDecoder, DecodeHint, Decoder};

use crate::context::Context;
use crate::de::Deserialize;

impl<'de, P: Decoder<'de>> Deserialize<'de, P> for String {
    fn deserialize(p: AnyDecoder<'_, 'de, P>, _ctx: &mut Context) -> anyhow::Result<Self> {
        Ok(p.decode(DecodeHint::String)?
            .try_into_string()?
            .into_owned())
    }
}
