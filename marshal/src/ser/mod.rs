use marshal_core::encode::Writer;

use crate::context::Context;

mod tuple;
mod vec;
mod map;
mod number;
mod string;
mod never;
mod option;

pub trait Serialize<W: Writer> {
    fn serialize(&self, w: W::AnyWriter<'_>, ctx: &mut Context) -> anyhow::Result<()>;
}
