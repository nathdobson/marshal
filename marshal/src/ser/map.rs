use std::collections::{BTreeMap, HashMap};
use std::hash::Hash;

use marshal_core::encode::{AnyEncoder, EntryEncoder, MapEncoder, Encoder};

use crate::context::Context;
use crate::ser::Serialize;

impl<W: Encoder, K: Eq + Hash + Serialize<W>, V: Serialize<W>> Serialize<W> for HashMap<K, V> {
    fn serialize(&self, w: W::AnyEncoder<'_>, ctx: &mut Context) -> anyhow::Result<()> {
        let mut w = w.encode_map(Some(self.len()))?;
        for (k, v) in self.iter() {
            let mut w = w.encode_entry()?;
            k.serialize(w.encode_key()?, ctx)?;
            v.serialize(w.encode_value()?, ctx)?;
            w.end()?;
        }
        w.end()?;
        Ok(())
    }
}

impl<W: Encoder, K: Ord + Serialize<W>, V: Serialize<W>> Serialize<W> for BTreeMap<K, V> {
    fn serialize(&self, w: W::AnyEncoder<'_>, ctx: &mut Context) -> anyhow::Result<()> {
        let mut w = w.encode_map(Some(self.len()))?;
        for (k, v) in self.iter() {
            let mut w = w.encode_entry()?;
            k.serialize(w.encode_key()?, ctx)?;
            v.serialize(w.encode_value()?, ctx)?;
            w.end()?;
        }
        w.end()?;
        Ok(())
    }
}
