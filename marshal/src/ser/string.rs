use crate::context::Context;
use crate::ser::Serialize;
use marshal_core::write::{AnyWriter, Writer};

impl<W: Writer> Serialize<W> for String {
    fn serialize(&self, w: W::AnyWriter<'_>, _ctx: &mut Context) -> anyhow::Result<()> {
        w.write_str(self)
    }
}
