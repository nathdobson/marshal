use crate::context::Context;
use crate::ser::Serialize;
use marshal_core::encode::{AnyWriter, SomeWriter, Writer};

impl<W: Writer, T: Serialize<W>> Serialize<W> for Option<T> {
    fn serialize(&self, w: W::AnyWriter<'_>, ctx: &mut Context) -> anyhow::Result<()> {
        match self {
            None => w.write_none(),
            Some(x) => {
                let mut w = w.write_some()?;
                x.serialize(w.write_some()?, ctx)?;
                w.end()?;
                Ok(())
            }
        }
    }
}
