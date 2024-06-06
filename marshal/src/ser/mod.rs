mod tuple;
mod vec;
mod map;

use crate::context::Context;
use marshal_core::write::Writer;

pub trait Serialize<W: Writer> {
    fn serialize(&self, w: W::AnyWriter<'_>, ctx: &mut Context) -> anyhow::Result<()>;
}
