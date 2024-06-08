use std::collections::{BTreeMap, HashMap};
use std::hash::Hash;

use marshal_core::encode::{AnyWriter, EntryWriter, MapWriter, Writer};

use crate::context::Context;
use crate::ser::Serialize;

impl<W: Writer, K: Eq + Hash + Serialize<W>, V: Serialize<W>> Serialize<W> for HashMap<K, V> {
    fn serialize(&self, w: W::AnyWriter<'_>, ctx: &mut Context) -> anyhow::Result<()> {
        let mut w = w.write_map(Some(self.len()))?;
        for (k, v) in self.iter() {
            let mut w = w.write_entry()?;
            k.serialize(w.write_key()?, ctx)?;
            v.serialize(w.write_value()?, ctx)?;
            w.end()?;
        }
        w.end()?;
        Ok(())
    }
}

impl<W: Writer, K: Ord + Serialize<W>, V: Serialize<W>> Serialize<W> for BTreeMap<K, V> {
    fn serialize(&self, w: W::AnyWriter<'_>, ctx: &mut Context) -> anyhow::Result<()> {
        let mut w = w.write_map(Some(self.len()))?;
        for (k, v) in self.iter() {
            let mut w = w.write_entry()?;
            k.serialize(w.write_key()?, ctx)?;
            v.serialize(w.write_value()?, ctx)?;
            w.end()?;
        }
        w.end()?;
        Ok(())
    }
}
