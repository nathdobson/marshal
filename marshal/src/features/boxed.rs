use marshal_core::decode::{AnyDecoder, Decoder};

use crate::context::Context;
use crate::de::Deserialize;

use marshal_core::encode::{AnyEncoder, Encoder};
use marshal_pointer::boxed::BoxRef;

use crate::ser::Serialize;

impl<D: Decoder, T: Deserialize<D>> Deserialize<D> for Box<T> {
    fn deserialize<'p, 'de>(p: AnyDecoder<'p, 'de, D>, ctx: Context) -> anyhow::Result<Self> {
        Ok(Box::new(T::deserialize(p, ctx)?))
    }
}

impl<E: Encoder, T: Serialize<E>> Serialize<E> for Box<T> {
    fn serialize<'w, 'en>(&self, w: AnyEncoder<'w, 'en, E>, ctx: Context) -> anyhow::Result<()> {
        (**self).serialize(w, ctx)
    }
}

impl<E: Encoder, T: Serialize<E>> Serialize<E> for BoxRef<T> {
    fn serialize<'w, 'en>(&self, w: AnyEncoder<'w, 'en, E>, ctx: Context) -> anyhow::Result<()> {
        (**self).serialize(w, ctx)
    }
}
