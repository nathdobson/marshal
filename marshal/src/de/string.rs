use marshal_core::decode::{AnyGenDecoder, DecodeHint,  GenDecoder};

use crate::context::Context;
use crate::de::Deserialize;

impl<D: GenDecoder> Deserialize<D> for String {
    fn deserialize<'p, 'de>(d: AnyGenDecoder<'p, 'de, D>, _ctx: Context) -> anyhow::Result<Self> {
        Ok(d.decode(DecodeHint::String)?
            .try_into_string()?
            .into_owned())
    }
}
