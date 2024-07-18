use std::borrow::Borrow;
use std::collections::{hash_map, HashMap, HashSet};
use std::fmt::{Debug, Formatter};
use std::hash::Hash;

use marshal::context::Context;
use marshal::de::Deserialize;
use marshal::decode::{AnyDecoder, DecodeHint, Decoder};
use marshal::encode::{AnyEncoder, Encoder};
use marshal::ser::Serialize;

use crate::de::DeserializeUpdate;
use crate::ser::set_channel::{SetPublisher, SetSubscriber};
use crate::ser::{SerializeStream, SerializeUpdate};

pub struct UpdateHashMap<K, V> {
    map: HashMap<K, V>,
    publisher: SetPublisher<HashSet<K>>,
}

pub struct UpdateHashMapStream<K, VS> {
    subscriber: SetSubscriber<HashSet<K>>,
    streams: HashMap<K, VS>,
}

pub struct OccupiedEntry<'a, K: 'a, V: 'a> {
    inner: hash_map::OccupiedEntry<'a, K, V>,
    publisher: &'a mut SetPublisher<HashSet<K>>,
}

pub struct VacantEntry<'a, K: 'a, V: 'a> {
    inner: hash_map::VacantEntry<'a, K, V>,
    publisher: &'a mut SetPublisher<HashSet<K>>,
}

pub enum Entry<'a, K: 'a, V: 'a> {
    Occupied(OccupiedEntry<'a, K, V>),
    Vacant(VacantEntry<'a, K, V>),
}

impl<K: Eq + Hash + Sync + Send + Clone, V: SerializeStream> SerializeStream
    for UpdateHashMap<K, V>
{
    type Stream = UpdateHashMapStream<K, V::Stream>;

    fn start_stream(&self, mut ctx: Context) -> anyhow::Result<Self::Stream> {
        Ok(UpdateHashMapStream {
            subscriber: self.publisher.subscribe(),
            streams: self
                .map
                .iter()
                .map(|(k, v)| Ok((k.clone(), v.start_stream(ctx.reborrow())?)))
                .collect::<anyhow::Result<_>>()?,
        })
    }
}

impl<E: Encoder, K: Eq + Hash + Sync + Send + Serialize<E>, V: Serialize<E>> Serialize<E>
    for UpdateHashMap<K, V>
{
    fn serialize<'w, 'en>(&self, e: AnyEncoder<'w, 'en, E>, ctx: Context) -> anyhow::Result<()> {
        self.map.serialize(e, ctx)
    }
}

impl<E: Encoder, K: Eq + Hash + Sync + Send + Clone + Serialize<E>, V: SerializeUpdate<E>>
    SerializeUpdate<E> for UpdateHashMap<K, V>
{
    fn serialize_update<'w, 'en>(
        &self,
        stream: &mut Self::Stream,
        e: AnyEncoder<'w, 'en, E>,
        mut ctx: Context,
    ) -> anyhow::Result<()> {
        let ref mut queue = *stream.subscriber.recv();
        let mut e = e.encode_map(Some(queue.len()))?;
        for key in queue.drain() {
            let mut e = e.encode_entry()?;
            key.serialize(e.encode_key()?, ctx.reborrow())?;
            let source = self.map.get(&key);
            if let Some(source) = source {
                let mut e = e.encode_value()?.encode_some()?;
                match stream.streams.entry(key) {
                    hash_map::Entry::Occupied(mut o) => {
                        source.serialize_update(o.get_mut(), e.encode_some()?, ctx.reborrow())?;
                    }
                    hash_map::Entry::Vacant(v) => {
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

impl<D: Decoder, K: Eq + Hash + Deserialize<D>, V: Deserialize<D>> Deserialize<D>
    for UpdateHashMap<K, V>
{
    fn deserialize<'p, 'de>(d: AnyDecoder<'p, 'de, D>, ctx: Context) -> anyhow::Result<Self> {
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

impl<D: Decoder, K: Eq + Hash + Deserialize<D>, V: DeserializeUpdate<D>> DeserializeUpdate<D>
    for UpdateHashMap<K, V>
{
    fn deserialize_update<'p, 'de>(
        &mut self,
        d: AnyDecoder<'p, 'de, D>,
        mut ctx: Context,
    ) -> anyhow::Result<()> {
        let mut d = d.decode(DecodeHint::Map)?.try_into_map()?;
        while let Some(mut d) = d.decode_next()? {
            let key = K::deserialize(d.decode_key()?, ctx.reborrow())?;
            let v = d.decode_value()?;
            if let Some(mut v) = v.decode(DecodeHint::Option)?.try_into_option()? {
                match self.map.entry(key) {
                    hash_map::Entry::Occupied(mut o) => {
                        o.get_mut()
                            .deserialize_update(v.decode_some()?, ctx.reborrow())?;
                    }
                    hash_map::Entry::Vacant(vac) => {
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
    pub fn get<Q>(&self, k: &Q) -> Option<&V>
    where
        Q: ?Sized + Hash + Eq,
        K: Borrow<Q>,
    {
        self.map.get(k)
    }
    pub fn get_mut(&mut self, k: &K) -> Option<&mut V> {
        let result = self.map.get_mut(k)?;
        self.publisher.send(&k);
        Some(result)
    }
    pub fn entry(&mut self, k: K) -> Entry<K, V> {
        match self.map.entry(k) {
            hash_map::Entry::Occupied(o) => Entry::Occupied(OccupiedEntry {
                inner: o,
                publisher: &mut self.publisher,
            }),
            hash_map::Entry::Vacant(v) => Entry::Vacant(VacantEntry {
                inner: v,
                publisher: &mut self.publisher,
            }),
        }
    }
    pub fn iter<'a>(&'a self) -> impl 'a + Iterator<Item = (&'a K, &'a V)> {
        self.map.iter()
    }
}

impl<'a, K: 'a + Eq + Hash + Sync + Send + Clone, V: 'a> Entry<'a, K, V> {
    pub fn or_default(self) -> &'a V
    where
        V: Default,
    {
        self.or_insert_with(V::default)
    }
    pub fn or_default_mut(self) -> &'a mut V
    where
        V: Default,
    {
        self.or_insert_with_mut(V::default)
    }
    pub fn or_insert_with<F: FnOnce() -> V>(self, f: F) -> &'a V {
        match self {
            Entry::Occupied(o) => o.into_ref(),
            Entry::Vacant(v) => v.insert(f()),
        }
    }
    pub fn or_insert_with_mut<F: FnOnce() -> V>(self, f: F) -> &'a mut V {
        match self {
            Entry::Occupied(o) => o.into_mut(),
            Entry::Vacant(v) => v.insert(f()),
        }
    }
}

impl<'a, K: 'a + Eq + Hash + Sync + Send + Clone, V: 'a> VacantEntry<'a, K, V> {
    pub fn insert(self, value: V) -> &'a mut V {
        self.publisher.send(self.inner.key());
        self.inner.insert(value)
    }
}

impl<'a, K: 'a + Eq + Hash + Sync + Send + Clone, V: 'a> OccupiedEntry<'a, K, V> {
    pub fn into_ref(self) -> &'a V {
        self.inner.into_mut()
    }
    pub fn into_mut(self) -> &'a mut V {
        self.publisher.send(self.inner.key());
        self.inner.into_mut()
    }
}

impl<K: Debug, V: Debug> Debug for UpdateHashMap<K, V> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        self.map.fmt(f)
    }
}

impl<K: Sync + Send + Eq + Hash + Clone, V> Default for UpdateHashMap<K, V> {
    fn default() -> Self {
        UpdateHashMap::new()
    }
}
