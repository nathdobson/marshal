use crate::context::Context;
use crate::de::Deserialize;
use crate::ser::Serialize;
use marshal_core::decode::{AnyDecoder, Decoder};
use marshal_core::encode::{AnyEncoder, Encoder};
use ordered_float::OrderedFloat;

impl<E: Encoder, T: Serialize<E>> Serialize<E> for OrderedFloat<T> {
    fn serialize<'w, 'en>(&self, e: AnyEncoder<'w, 'en, E>, ctx: Context) -> anyhow::Result<()> {
        self.0.serialize(e, ctx)
    }
}

impl<D: Decoder, T: Deserialize<D>> Deserialize<D> for OrderedFloat<T> {
    fn deserialize<'p, 'de>(d: AnyDecoder<'p, 'de, D>, ctx: Context) -> anyhow::Result<Self> {
        Ok(OrderedFloat(T::deserialize(d, ctx)?))
    }
}
