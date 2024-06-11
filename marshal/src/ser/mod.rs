use marshal_core::encode::Encoder;

use crate::context::Context;

mod map;
mod never;
mod number;
mod option;
mod string;
mod tuple;
mod vec;
mod boxed;
pub mod rc;

pub trait Serialize<W: Encoder> {
    fn serialize(&self, w: W::AnyEncoder<'_>, ctx: &mut Context) -> anyhow::Result<()>;
}

fn is_object_safe<W: Encoder, T: Serialize<W>>(x: &T) -> &dyn Serialize<W> {
    x
}
