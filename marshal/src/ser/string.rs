use marshal_core::encode::{AnyEncoder, Encoder};

use crate::context::Context;
use crate::ser::Serialize;

impl<W: Encoder> Serialize<W> for String {
    fn serialize<'w, 'en>(
        &self,
        w: AnyEncoder<'w, 'en, W>,
        _ctx: Context,
    ) -> anyhow::Result<()> {
        w.encode_str(self)
    }
}
