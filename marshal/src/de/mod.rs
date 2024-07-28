use marshal_core::decode::{AnyDecoder, Decoder};

use crate::context::Context;

pub mod rc;

pub trait Deserialize<D: Decoder> {
    fn deserialize<'p, 'de>(d: AnyDecoder<'p, 'de, D>, ctx: Context) -> anyhow::Result<Self>
    where
        Self: Sized;
}

fn is_object_safe<D: Decoder, T: Deserialize<D>>(x: &T) -> &dyn Deserialize<D> {
    x
}

