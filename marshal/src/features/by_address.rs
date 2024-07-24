use crate::context::Context;
use crate::ser::Serialize;
use by_address::ByAddress;
use marshal_core::encode::{AnyEncoder, Encoder};
use std::ops::Deref;

impl<E: Encoder, T: Deref + Serialize<E>> Serialize<E> for ByAddress<T> {
    fn serialize<'w, 'en>(&self, e: AnyEncoder<'w, 'en, E>, ctx: Context) -> anyhow::Result<()> {
        self.0.serialize(e, ctx)
    }
}
use crate::de::Deserialize;
use marshal_core::decode::{AnyDecoder, Decoder};

impl<D: Decoder, T: Deref + Deserialize<D>> Deserialize<D> for ByAddress<T> {
    fn deserialize<'p, 'de>(d: AnyDecoder<'p, 'de, D>, ctx: Context) -> anyhow::Result<Self> {
        Ok(ByAddress(T::deserialize(d, ctx)?))
    }
}
