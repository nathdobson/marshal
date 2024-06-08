use marshal_core::encode::Encoder;
use crate::context::Context;
use crate::ser::Serialize;

impl<W: Encoder> Serialize<W> for ! {
    fn serialize(&self, _: W::AnyEncoder<'_>, _ctx: &mut Context) -> anyhow::Result<()> {
        *self
    }
}