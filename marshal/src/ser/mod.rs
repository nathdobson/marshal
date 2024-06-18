use marshal_core::encode::{AnyEncoder, Encoder};

use crate::context::Context;

mod boxed;
mod map;
mod never;
mod number;
mod option;
pub mod rc;
mod reference;
mod string;
mod tuple;
mod vec;

pub trait Serialize<W: Encoder> {
    fn serialize(&self, w: AnyEncoder<'_, W>, ctx: &mut Context) -> anyhow::Result<()>;
}

fn is_object_safe<W: Encoder, T: Serialize<W>>(x: &T) -> &dyn Serialize<W> {
    x
}
