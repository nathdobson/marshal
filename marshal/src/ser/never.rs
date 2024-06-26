use marshal_core::encode::{AnyEncoder, Encoder};

use crate::context::Context;
use crate::ser::Serialize;

impl<W: Encoder> Serialize<W> for ! {
    fn serialize(&self, _: AnyEncoder<'_, W>, _ctx: Context) -> anyhow::Result<()> {
        *self
    }
}
