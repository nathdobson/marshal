use marshal_core::write::Writer;

use crate::context::Context;

mod tuple;
mod vec;
mod map;

pub trait Serialize<W: Writer> {
    fn serialize(&self, w: W::AnyWriter<'_>, ctx: &mut Context) -> anyhow::Result<()>;
}
