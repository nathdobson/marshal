use marshal_core::decode::{AnyDecoder, Decoder};

use crate::context::Context;
use crate::de::Deserialize;

impl<'de, D: Decoder, T: Deserialize<D>> Deserialize<D> for Box<T> {
    fn deserialize<'p>(p: AnyDecoder<'p, D>, ctx: Context) -> anyhow::Result<Self> {
        Ok(Box::new(T::deserialize(p, ctx)?))
    }
}
