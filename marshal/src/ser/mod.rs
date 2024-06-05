mod tuple;
mod vec;

use crate::context::Context;
use crate::write::Writer;

pub trait Serialize<W: Writer> {
    fn serialize(&self, w: W::AnyWriter<'_>, ctx: &mut Context) -> anyhow::Result<()>;
}
