use std::ops::Deref;
use crate::context::Context;
use crate::de::Deserialize;
use by_address::ByAddress;
use marshal_core::decode::{AnyDecoder, Decoder};

impl<D: Decoder, T: Deref + Deserialize<D>> Deserialize<D> for ByAddress<T> {
    fn deserialize<'p, 'de>(d: AnyDecoder<'p, 'de, D>, ctx: Context) -> anyhow::Result<Self> {
        Ok(ByAddress(T::deserialize(d, ctx)?))
    }
}
