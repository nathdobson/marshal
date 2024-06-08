use marshal_core::encode::Encoder;

use crate::context::Context;

mod tuple;
mod vec;
mod map;
mod number;
mod string;
mod never;
mod option;

pub trait Serialize<W: Encoder> {
    fn serialize(&self, w: W::AnyEncoder<'_>, ctx: &mut Context) -> anyhow::Result<()>;
}
