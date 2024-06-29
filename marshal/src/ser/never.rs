use marshal_core::encode::{AnyEncoder, AnyGenEncoder, Encoder, GenEncoder};

use crate::context::Context;
use crate::ser::Serialize;

impl<W: GenEncoder> Serialize<W> for ! {
    fn serialize<'w, 'en>(&self, _: AnyGenEncoder<'w, 'en, W>, _ctx: Context) -> anyhow::Result<()> {
        *self
    }
}
