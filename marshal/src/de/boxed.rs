use crate::context::Context;
use crate::de::Deserialize;
use marshal_core::decode::Decoder;

impl<'de, D: Decoder<'de>, T: Deserialize<'de, D>> Deserialize<'de, D> for Box<T> {
    fn deserialize<'p>(p: D::AnyDecoder<'p>, ctx: &mut Context) -> anyhow::Result<Self> {
        Ok(Box::new(T::deserialize(p, ctx)?))
    }
}
