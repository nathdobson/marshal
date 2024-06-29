use marshal_core::encode::{AnyEncoder, Encoder};

use crate::context::Context;
use crate::ser::Serialize;

impl<W: Encoder, T: Serialize<W>> Serialize<W> for Option<T> {
    fn serialize<'w, 'en>(&self, w: AnyEncoder<'w, 'en, W>, ctx: Context) -> anyhow::Result<()> {
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
