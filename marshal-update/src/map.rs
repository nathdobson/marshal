use crate::de::DeserializeUpdate;
use crate::ser::set_channel::{SetPublisher, SetSubscriber};
use crate::ser::{SerializeStream, SerializeUpdate};
use atomic_refcell::AtomicRefCell;
use marshal::context::Context;
use marshal::de::Deserialize;
use marshal::decode::{AnyDecoder, DecodeHint, Decoder};
use marshal::encode::{AnyEncoder, Encoder};
use marshal::ser::Serialize;
use std::collections::hash_map::Entry;
use std::collections::{HashMap, HashSet};
use std::hash::Hash;
use std::sync::{Arc, Weak};
use weak_table::PtrWeakHashSet;

pub struct UpdateHashMap<K, V> {
    map: HashMap<K, V>,
    publisher: SetPublisher<HashSet<K>>,
}

pub struct UpdateHashMapStream<K, VS> {
    subscriber: SetSubscriber<HashSet<K>>,
    streams: HashMap<K, VS>,
}

impl<K: Eq + Hash + Sync + Send + Clone, V: SerializeStream> SerializeStream
    for UpdateHashMap<K, V>
{
    type Stream = UpdateHashMapStream<K, V::Stream>;

    fn start_stream(&self, ctx: &mut Context) -> anyhow::Result<Self::Stream> {
        Ok(UpdateHashMapStream {
            subscriber: self.publisher.subscribe(),
            streams: self
                .map
                .iter()
                .map(|(k, v)| Ok((k.clone(), v.start_stream(ctx)?)))
                .collect::<anyhow::Result<_>>()?,
        })
    }
}

impl<E: Encoder, K: Eq + Hash + Sync + Send + Serialize<E>, V: Serialize<E>> Serialize<E>
    for UpdateHashMap<K, V>
{
    fn serialize(&self, e: AnyEncoder<'_, E>, ctx: &mut Context) -> anyhow::Result<()> {
        self.map.serialize(e, ctx)
    }
}

impl<E: Encoder, K: Eq + Hash + Sync + Send + Clone + Serialize<E>, V: SerializeUpdate<E>>
    SerializeUpdate<E> for UpdateHashMap<K, V>
{
    fn serialize_update(
        &self,
        stream: &mut Self::Stream,
        e: AnyEncoder<E>,
        ctx: &mut Context,
    ) -> anyhow::Result<()> {
        let ref mut queue = *stream.subscriber.recv();
        let mut e = e.encode_map(Some(queue.len()))?;
        for key in queue.drain() {
            let mut e = e.encode_entry()?;
            key.serialize(e.encode_key()?, ctx)?;
            let source = self.map.get(&key);
            if let Some(source) = source {
                let mut e = e.encode_value()?.encode_some()?;
                match stream.streams.entry(key) {
                    Entry::Occupied(mut o) => {
                        source.serialize_update(o.get_mut(), e.encode_some()?, ctx)?;
                    }
                    Entry::Vacant(v) => {
                        v.insert(source.start_stream(ctx)?);
                        source.serialize(e.encode_some()?, ctx)?;
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

impl<'de, D: Decoder<'de>, K: Eq + Hash + Deserialize<'de, D>, V: Deserialize<'de, D>>
    Deserialize<'de, D> for UpdateHashMap<K, V>
{
    fn deserialize<'p>(d: AnyDecoder<'p, 'de, D>, ctx: &mut Context) -> anyhow::Result<Self> {
        Ok(Self::from(HashMap::deserialize(d, ctx)?))
    }
}

impl<K, V> From<HashMap<K, V>> for UpdateHashMap<K, V> {
    fn from(map: HashMap<K, V>) -> Self {
        UpdateHashMap {
            map,
            publisher: SetPublisher::new(),
        }
    }
}

impl<'de, D: Decoder<'de>, K: Eq + Hash + Deserialize<'de, D>, V: DeserializeUpdate<'de, D>>
    DeserializeUpdate<'de, D> for UpdateHashMap<K, V>
{
    fn deserialize_update<'p>(
        &mut self,
        d: AnyDecoder<'p, 'de, D>,
        ctx: &mut Context,
    ) -> anyhow::Result<()> {
        let mut d = d.decode(DecodeHint::Map)?.try_into_map()?;
        while let Some(mut d) = d.decode_next()? {
            let key = K::deserialize(d.decode_key()?, ctx)?;
            let v = d.decode_value()?;
            if let Some(mut v) = v.decode(DecodeHint::Option)?.try_into_option()? {
                match self.map.entry(key) {
                    Entry::Occupied(mut o) => {
                        o.get_mut().deserialize_update(v.decode_some()?, ctx)?;
                    }
                    Entry::Vacant(vac) => {
                        vac.insert(V::deserialize(v.decode_some()?, ctx)?);
                    }
                }
                v.decode_end()?;
            }
        }
        Ok(())
    }
}

impl<K: Eq + Hash + Sync + Send + Clone, V> UpdateHashMap<K, V> {
    pub fn new() -> Self {
        UpdateHashMap {
            map: HashMap::new(),
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