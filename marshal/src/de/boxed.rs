use marshal_core::decode::{AnyGenDecoder, GenDecoder};

use crate::context::Context;
use crate::de::Deserialize;

impl<D: GenDecoder, T: Deserialize<D>> Deserialize<D> for Box<T> {
    fn deserialize<'p, 'de>(p: AnyGenDecoder<'p, 'de, D>, ctx: Context) -> anyhow::Result<Self> {
        Ok(Box::new(T::deserialize(p, ctx)?))
    }
}
