use marshal_core::decode::{AnyDecoder, Decoder};

use crate::context::Context;
use crate::de::Deserialize;

impl<D: Decoder, T: Deserialize<D>> Deserialize<D> for Box<T> {
    fn deserialize<'p, 'de>(p: AnyDecoder<'p, 'de, D>, ctx: Context) -> anyhow::Result<Self> {
        Ok(Box::new(T::deserialize(p, ctx)?))
    }
}
