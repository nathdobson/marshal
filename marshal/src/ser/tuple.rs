use marshal_core::Primitive;
use marshal_core::write::{AnyWriter, TupleWriter, Writer};

use crate::context::Context;
use crate::ser::Serialize;

impl<W: Writer> Serialize<W> for () {
    fn serialize(&self, w: W::AnyWriter<'_>, _ctx: &mut Context) -> anyhow::Result<()> {
        w.write_prim(Primitive::Unit)
    }
}

impl<W: Writer, T1: Serialize<W>> Serialize<W> for (T1,) {
    fn serialize(&self, w: W::AnyWriter<'_>, ctx: &mut Context) -> anyhow::Result<()> {
        let mut w = w.write_tuple(1)?;
        self.0.serialize(w.write_element()?, ctx)?;
        w.end()?;
        Ok(())
    }
}

impl<W: Writer, T1: Serialize<W>, T2: Serialize<W>> Serialize<W> for (T1, T2) {
    fn serialize(&self, w: W::AnyWriter<'_>, ctx: &mut Context) -> anyhow::Result<()> {
        let mut w = w.write_tuple(2)?;
        self.0.serialize(w.write_element()?, ctx)?;
        self.1.serialize(w.write_element()?, ctx)?;
        w.end()?;
        Ok(())
    }
}
