use marshal_core::encode::{AnyEncoder, AnyGenEncoder, Encoder, GenEncoder};

use crate::context::Context;
use crate::ser::Serialize;

impl<W: GenEncoder, T: Serialize<W>> Serialize<W> for Option<T> {
    fn serialize<'w, 'en>(&self, w: AnyGenEncoder<'w, 'en, W>, ctx: Context) -> anyhow::Result<()> {
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
