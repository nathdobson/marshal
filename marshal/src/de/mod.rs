use marshal_core::decode::{AnyDecoder, Decoder};

use crate::context::Context;

mod boxed;
mod map;
mod never;
mod number;
mod option;
pub mod rc;
mod string;
mod tuple;
mod vec;
mod result;
mod anyhow_de;

pub trait Deserialize<D: Decoder> {
    fn deserialize<'p, 'de>(d: AnyDecoder<'p, 'de, D>, ctx: Context) -> anyhow::Result<Self>
    where
        Self: Sized;
}

fn is_object_safe<D: Decoder, T: Deserialize<D>>(x: &T) -> &dyn Deserialize<D> {
    x
}

