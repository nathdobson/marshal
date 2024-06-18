use marshal_core::encode::{AnyEncoder, Encoder};

use crate::context::Context;
use crate::ser::Serialize;

impl<W: Encoder> Serialize<W> for String {
    fn serialize(&self, w: AnyEncoder<'_, W>, _ctx: &mut Context) -> anyhow::Result<()> {
        w.encode_str(self)
    }
}
