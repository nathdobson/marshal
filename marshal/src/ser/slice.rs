use crate::context::Context;
use crate::ser::Serialize;
use marshal_core::encode::{AnyEncoder, Encoder};

impl<E: Encoder, T: Serialize<E>> Serialize<E> for [T] {
    default fn serialize<'w, 'en>(
        &self,
        e: AnyEncoder<'w, 'en, E>,
        mut ctx: Context,
    ) -> anyhow::Result<()> {
        let mut e = e.encode_seq(self.len())?;
        for x in self {
            x.serialize(e.encode_element()?, ctx.reborrow())?;
        }
        e.end()?;
        Ok(())
    }
}

impl<E: Encoder> Serialize<E> for [u8] {
    fn serialize<'w, 'en>(&self, e: AnyEncoder<'w, 'en, E>, _ctx: Context) -> anyhow::Result<()> {
        e.encode_bytes(self)?;
        Ok(())
    }
}
