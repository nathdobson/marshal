use crate::context::Context;
use crate::ser::Serialize;
use crate::write::{AnyWriter, SeqWriter, Writer};

impl<W: Writer, T: Serialize<W>> Serialize<W> for Vec<T> {
    fn serialize(&self, w: W::AnyWriter<'_>, ctx: &mut Context) -> anyhow::Result<()> {
        let mut w = w.write_seq(Some(self.len()))?;
        for x in self.iter() {
            x.serialize(w.write_element()?, ctx)?;
        }
        w.end()?;
        Ok(())
    }
}