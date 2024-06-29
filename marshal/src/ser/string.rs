use marshal_core::encode::{AnyEncoder, AnyGenEncoder, Encoder, GenEncoder};

use crate::context::Context;
use crate::ser::Serialize;

impl<W: GenEncoder> Serialize<W> for String {
    fn serialize<'w, 'en>(
        &self,
        w: AnyGenEncoder<'w, 'en, W>,
        _ctx: Context,
    ) -> anyhow::Result<()> {
        w.encode_str(self)
    }
}
