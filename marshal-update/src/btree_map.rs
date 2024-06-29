use std::collections::btree_map::Entry;
use std::collections::{BTreeMap, BTreeSet};
use std::fmt::{Debug, Formatter};
use std::mem;

use marshal::context::Context;
use marshal::de::Deserialize;
use marshal::decode::{AnyGenDecoder, DecodeHint, GenDecoder};
use marshal::encode::{AnyEncoder, AnyGenEncoder, Encoder, GenEncoder};
use marshal::ser::Serialize;

use crate::de::DeserializeUpdate;
use crate::ser::set_channel::{SetPublisher, SetSubscriber};
use crate::ser::{SerializeStream, SerializeUpdate};

pub struct UpdateBTreeMap<K, V> {
    map: BTreeMap<K, V>,
    publisher: SetPublisher<BTreeSet<K>>,
}

pub struct UpdateBTreeMapStream<K, VS> {
    subscriber: SetSubscriber<BTreeSet<K>>,
    streams: BTreeMap<K, VS>,
}

impl<K: Ord + Sync + Send + Clone, V: SerializeStream> SerializeStream for UpdateBTreeMap<K, V> {
    type Stream = UpdateBTreeMapStream<K, V::Stream>;

    fn start_stream(&self, mut ctx: Context) -> anyhow::Result<Self::Stream> {
        Ok(UpdateBTreeMapStream {
            subscriber: self.publisher.subscribe(),
            streams: self
                .map
                .iter()
                .map(|(k, v)| Ok((k.clone(), v.start_stream(ctx.reborrow())?)))
                .collect::<anyhow::Result<_>>()?,
        })
    }
}

impl<E: GenEncoder, K: Ord + Sync + Send + Serialize<E>, V: Serialize<E>> Serialize<E>
    for UpdateBTreeMap<K, V>
{
    fn serialize<'w, 'en>(&self, e: AnyGenEncoder<'w, 'en, E>, ctx: Context) -> anyhow::Result<()> {
        self.map.serialize(e, ctx)
    }
}

impl<E: GenEncoder, K: Ord + Sync + Send + Clone + Serialize<E>, V: SerializeUpdate<E>>
    SerializeUpdate<E> for UpdateBTreeMap<K, V>
{
    fn serialize_update<'w, 'en>(
        &self,
        stream: &mut Self::Stream,
        e: AnyGenEncoder<'w, 'en, E>,
        mut ctx: Context,
    ) -> anyhow::Result<()> {
        let queue = mem::replace(&mut *stream.subscriber.recv(), BTreeSet::new());
        let mut e = e.encode_map(Some(queue.len()))?;
        for key in queue {
            let mut e = e.encode_entry()?;
            key.serialize(e.encode_key()?, ctx.reborrow())?;
            let source = self.map.get(&key);
            if let Some(source) = source {
                let mut e = e.encode_value()?.encode_some()?;
                match stream.streams.entry(key) {
                    Entry::Occupied(mut o) => {
                        source.serialize_update(o.get_mut(), e.encode_some()?, ctx.reborrow())?;
                    }
                    Entry::Vacant(v) => {
                        v.insert(source.start_stream(ctx.reborrow())?);
                        source.serialize(e.encode_some()?, ctx.reborrow())?;
                    }
                }
                e.end()?;
            } else {
                e.encode_value()?.encode_none()?;
                stream.streams.remove(&key);
            }
            e.end()?;
        }
        e.end()?;
        Ok(())
    }
}

impl<D: GenDecoder, K: Ord + Deserialize<D>, V: Deserialize<D>> Deserialize<D>
    for UpdateBTreeMap<K, V>
{
    fn deserialize<'p, 'de>(d: AnyGenDecoder<'p, 'de, D>, ctx: Context) -> anyhow::Result<Self> {
        Ok(Self::from(BTreeMap::deserialize(d, ctx)?))
    }
}

impl<K, V> From<BTreeMap<K, V>> for UpdateBTreeMap<K, V> {
    fn from(map: BTreeMap<K, V>) -> Self {
        UpdateBTreeMap {
            map,
            publisher: SetPublisher::new(),
        }
    }
}

impl<D: GenDecoder, K: Ord + Deserialize<D>, V: DeserializeUpdate<D>> DeserializeUpdate<D>
    for UpdateBTreeMap<K, V>
{
    fn deserialize_update<'p, 'de>(
        &mut self,
        d: AnyGenDecoder<'p, 'de, D>,
        mut ctx: Context,
    ) -> anyhow::Result<()> {
        let mut d = d.decode(DecodeHint::Map)?.try_into_map()?;
        while let Some(mut d) = d.decode_next()? {
            let key = K::deserialize(d.decode_key()?, ctx.reborrow())?;
            let v = d.decode_value()?;
            if let Some(mut v) = v.decode(DecodeHint::Option)?.try_into_option()? {
                match self.map.entry(key) {
                    Entry::Occupied(mut o) => {
                        o.get_mut()
                            .deserialize_update(v.decode_some()?, ctx.reborrow())?;
                    }
                    Entry::Vacant(vac) => {
                        vac.insert(V::deserialize(v.decode_some()?, ctx.reborrow())?);
                    }
                }
                v.decode_end()?;
            } else {
                self.map.remove(&key);
            }
        }
        Ok(())
    }
}

impl<K: Ord + Sync + Send + Clone, V> UpdateBTreeMap<K, V> {
    pub fn new() -> Self {
        UpdateBTreeMap {
            map: BTreeMap::new(),
            publisher: SetPublisher::new(),
        }
    }
    pub fn insert(&mut self, k: K, v: V) -> Option<V> {
        let result = self.map.insert(k.clone(), v);
        self.publisher.send(&k);
        result
    }
    pub fn remove(&mut self, k: &K) -> Option<V> {
        let result = self.map.remove(k)?;
        self.publisher.send(&k);
        Some(result)
    }
    pub fn get(&self, k: &K) -> Option<&V> {
        self.map.get(k)
    }
    pub fn get_mut(&mut self, k: &K) -> Option<&mut V> {
        let result = self.map.get_mut(k)?;
        self.publisher.send(&k);
        Some(result)
    }
}

impl<K: Debug, V: Debug> Debug for UpdateBTreeMap<K, V> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        self.map.fmt(f)
    }
}
