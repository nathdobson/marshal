use marshal_core::encode::{AnyEncoder, TupleEncoder, Encoder};
use marshal_core::Primitive;

use crate::context::Context;
use crate::ser::Serialize;

impl<W: Encoder> Serialize<W> for () {
    fn serialize(&self, w: W::AnyEncoder<'_>, _ctx: &mut Context) -> anyhow::Result<()> {
        w.encode_prim(Primitive::Unit)
    }
}

impl<W: Encoder, T1: Serialize<W>> Serialize<W> for (T1, ) {
    fn serialize(&self, w: W::AnyEncoder<'_>, ctx: &mut Context) -> anyhow::Result<()> {
        let mut w = w.encode_tuple(1)?;
        self.0.serialize(w.encode_element()?, ctx)?;
        w.end()?;
        Ok(())
    }
}

impl<W: Encoder, T1: Serialize<W>, T2: Serialize<W>> Serialize<W> for (T1, T2) {
    fn serialize(&self, w: W::AnyEncoder<'_>, ctx: &mut Context) -> anyhow::Result<()> {
        let mut w = w.encode_tuple(2)?;
        self.0.serialize(w.encode_element()?, ctx)?;
        self.1.serialize(w.encode_element()?, ctx)?;
        w.end()?;
        Ok(())
    }
}

impl<W: Encoder, T1: Serialize<W>, T2: Serialize<W>, T3: Serialize<W>> Serialize<W>
for (T1, T2, T3)
{
    fn serialize(&self, w: W::AnyEncoder<'_>, ctx: &mut Context) -> anyhow::Result<()> {
        let mut w = w.encode_tuple(3)?;
        self.0.serialize(w.encode_element()?, ctx)?;
        self.1.serialize(w.encode_element()?, ctx)?;
        self.2.serialize(w.encode_element()?, ctx)?;
        w.end()?;
        Ok(())
    }
}

impl<W: Encoder, T1: Serialize<W>, T2: Serialize<W>, T3: Serialize<W>, T4: Serialize<W>> Serialize<W>
for (T1, T2, T3, T4)
{
    fn serialize(&self, w: W::AnyEncoder<'_>, ctx: &mut Context) -> anyhow::Result<()> {
        let mut w = w.encode_tuple(4)?;
        self.0.serialize(w.encode_element()?, ctx)?;
        self.1.serialize(w.encode_element()?, ctx)?;
        self.2.serialize(w.encode_element()?, ctx)?;
        self.3.serialize(w.encode_element()?, ctx)?;
        w.end()?;
        Ok(())
    }
}
