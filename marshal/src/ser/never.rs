use marshal_core::encode::Writer;
use crate::context::Context;
use crate::ser::Serialize;

impl<W:Writer> Serialize<W> for !{
    fn serialize(&self, _: W::AnyWriter<'_>, _ctx: &mut Context) -> anyhow::Result<()> {
        *self
    }
}