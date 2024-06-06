use marshal_core::Primitive;
use crate::context::Context;
use crate::ser::Serialize;
use marshal_core::write::{AnyWriter, Writer};

impl<W: Writer> Serialize<W> for u32 {
    fn serialize(&self, w: W::AnyWriter<'_>, ctx: &mut Context) -> anyhow::Result<()> {
        w.write_prim(Primitive::U32(*self))
    }
}
