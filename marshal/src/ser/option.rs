use marshal_core::encode::{AnyEncoder, Encoder};

use crate::context::Context;
use crate::ser::Serialize;

impl<W: Encoder, T: Serialize<W>> Serialize<W> for Option<T> {
    fn serialize(&self, w: AnyEncoder<'_, W>, ctx: &mut Context) -> anyhow::Result<()> {
        match self {
            None => w.encode_none(),
            Some(x) => {
                let mut w = w.encode_some()?;
                x.serialize(w.encode_some()?, ctx)?;
                w.end()?;
                Ok(())
            }
        }
    }
}
