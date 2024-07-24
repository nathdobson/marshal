use std::collections::{BTreeMap, HashMap};
use std::hash::Hash;

use marshal_core::encode::{AnyEncoder, Encoder};

use crate::context::Context;
use crate::ser::Serialize;

impl<W: Encoder, K: Eq + Hash + Serialize<W>, V: Serialize<W>> Serialize<W> for HashMap<K, V> {
    fn serialize<'w, 'en>(
        &self,
        w: AnyEncoder<'w, 'en, W>,
        mut ctx: Context,
    ) -> anyhow::Result<()> {
        let mut w = w.encode_map(self.len())?;
        for (k, v) in self.iter() {
            let mut w = w.encode_entry()?;
            k.serialize(w.encode_key()?, ctx.reborrow())?;
            v.serialize(w.encode_value()?, ctx.reborrow())?;
            w.end()?;
        }
        w.end()?;
        Ok(())
    }
}

impl<W: Encoder, K: Ord + Serialize<W>, V: Serialize<W>> Serialize<W> for BTreeMap<K, V> {
    fn serialize<'w, 'en>(
        &self,
        w: AnyEncoder<'w, 'en, W>,
        mut ctx: Context,
    ) -> anyhow::Result<()> {
        let mut w = w.encode_map(self.len())?;
        for (k, v) in self.iter() {
            let mut w = w.encode_entry()?;
            k.serialize(w.encode_key()?, ctx.reborrow())?;
            v.serialize(w.encode_value()?, ctx.reborrow())?;
            w.end()?;
        }
        w.end()?;
        Ok(())
    }
}
