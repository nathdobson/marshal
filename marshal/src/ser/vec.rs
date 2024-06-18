use marshal_core::encode::{AnyEncoder, Encoder};

use crate::context::Context;
use crate::ser::Serialize;

impl<W: Encoder, T: Serialize<W>> Serialize<W> for Vec<T> {
    default fn serialize(&self, w: AnyEncoder<'_, W>, ctx: &mut Context) -> anyhow::Result<()> {
        let mut w = w.encode_seq(Some(self.len()))?;
        for x in self.iter() {
            x.serialize(w.encode_element()?, ctx)?;
        }
        w.end()?;
        Ok(())
    }
}

impl<W: Encoder> Serialize<W> for Vec<u8> {
    fn serialize(&self, w: AnyEncoder<'_, W>, _ctx: &mut Context) -> anyhow::Result<()> {
        w.encode_bytes(self)
    }
}
