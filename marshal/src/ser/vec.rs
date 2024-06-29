use marshal_core::encode::{AnyEncoder, Encoder};

use crate::context::Context;
use crate::ser::Serialize;

impl<W: Encoder, T: Serialize<W>> Serialize<W> for Vec<T> {
    default fn serialize<'w, 'en>(
        &self,
        w: AnyEncoder<'w, 'en, W>,
        mut ctx: Context,
    ) -> anyhow::Result<()> {
        let mut w = w.encode_seq(Some(self.len()))?;
        for x in self.iter() {
            x.serialize(w.encode_element()?, ctx.reborrow())?;
        }
        w.end()?;
        Ok(())
    }
}

impl<W: Encoder> Serialize<W> for Vec<u8> {
    fn serialize<'w, 'en>(
        &self,
        w: AnyEncoder<'w, 'en, W>,
        _ctx: Context,
    ) -> anyhow::Result<()> {
        w.encode_bytes(self)
    }
}
