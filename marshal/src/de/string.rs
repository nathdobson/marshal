use marshal_core::decode::{AnyDecoder, DecodeHint, Decoder};

use crate::context::Context;
use crate::de::Deserialize;

impl<P: Decoder> Deserialize<P> for String {
    fn deserialize(p: AnyDecoder<'_, P>, _ctx: Context) -> anyhow::Result<Self> {
        Ok(p.decode(DecodeHint::String)?
            .try_into_string()?
            .decode_cow()?
            .into_owned())
    }
}
