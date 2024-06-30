use std::any::TypeId;
use std::collections::{HashMap, HashSet};
use std::marker::Unsize;
use std::ops::CoerceUnsized;

use marshal::context::Context;
use marshal::de::Deserialize;
use marshal::decode::{AnyDecoder, DecodeHint, Decoder};
use marshal::encode::{AnyEncoder, Encoder};
use marshal::ser::Serialize;
use marshal_object::Object;
use marshal_pointer::{AsFlatRef, DerefRaw, DowncastRef, RawAny};

use crate::de::DeserializeUpdate;
use crate::ser::set_channel::{SetPublisher, SetSubscriber};
use crate::ser::{SerializeStream, SerializeUpdate};

pub struct ObjectMap<C: Object> {
    map: HashMap<TypeId, C::Pointer<C::Dyn>>,
    publisher: SetPublisher<HashSet<TypeId>>,
}

pub struct ObjectMapStream {
    subscriber: SetSubscriber<HashSet<TypeId>>,
}

impl<C: Object> ObjectMap<C> {
    pub fn new() -> Self {
        ObjectMap {
            map: HashMap::new(),
            publisher: SetPublisher::new(),
        }
    }
    pub fn get<T: 'static>(&self) -> Option<&<C::Pointer<T> as AsFlatRef>::FlatRef>
    where
        for<'a> &'a <C::Pointer<C::Dyn> as AsFlatRef>::FlatRef:
            CoerceUnsized<&'a <C::Pointer<dyn RawAny> as AsFlatRef>::FlatRef>,
        <C::Pointer<dyn RawAny> as AsFlatRef>::FlatRef:
            DowncastRef<<C::Pointer<T> as AsFlatRef>::FlatRef>,
    {
        let dyn_flat_ref: &<C::Pointer<C::Dyn> as AsFlatRef>::FlatRef =
            self.map.get(&TypeId::of::<T>())?.as_flat_ref();
        let any_flat_ref: &<C::Pointer<dyn RawAny> as AsFlatRef>::FlatRef = dyn_flat_ref;
        let flat_ref: &<C::Pointer<T> as AsFlatRef>::FlatRef = any_flat_ref.downcast_ref().unwrap();
        Some(flat_ref)
    }
    pub fn get_or_default<T: 'static>(&mut self) -> &<C::Pointer<T> as AsFlatRef>::FlatRef
    where
        C::Pointer<T>: Default,
        C::Pointer<T>: CoerceUnsized<C::Pointer<C::Dyn>>,
        <C::Pointer<C::Dyn> as AsFlatRef>::FlatRef:
            Unsize<<C::Pointer<dyn RawAny> as AsFlatRef>::FlatRef>,
        <C::Pointer<dyn RawAny> as AsFlatRef>::FlatRef: DowncastRef<<C::Pointer<T> as AsFlatRef>::FlatRef>,
    {
        (self
            .map
            .entry(TypeId::of::<T>())
            .or_insert_with(|| C::Pointer::<T>::default())
            .as_flat_ref() as &<C::Pointer<dyn RawAny> as AsFlatRef>::FlatRef)
            .downcast_ref()
            .unwrap()
    }
    pub fn insert(&mut self, value: C::Pointer<C::Dyn>)
    where
        C::Pointer<C::Dyn>: DerefRaw<RawTarget = C::Dyn>,
        C::Dyn: RawAny,
    {
        self.publisher.send(&value.deref_raw().raw_type_id());
        self.map.insert(value.deref_raw().raw_type_id(), value);
    }
}

impl<E: Encoder, C: Object> Serialize<E> for ObjectMap<C>
where
    C::Pointer<C::Dyn>: Serialize<E>,
{
    fn serialize<'w, 'en>(
        &self,
        e: AnyEncoder<'w, 'en, E>,
        mut ctx: Context,
    ) -> anyhow::Result<()> {
        let mut e = e.encode_seq(Some(self.map.len()))?;
        for x in self.map.values() {
            x.serialize(e.encode_element()?, ctx.reborrow())?;
        }
        e.end()?;
        Ok(())
    }
}

impl<C: Object> SerializeStream for ObjectMap<C> {
    type Stream = ObjectMapStream;

    fn start_stream(&self, _ctx: Context) -> anyhow::Result<Self::Stream> {
        Ok(ObjectMapStream {
            subscriber: self.publisher.subscribe(),
        })
    }
}

impl<E: Encoder, C: Object> SerializeUpdate<E> for ObjectMap<C>
where
    C::Pointer<C::Dyn>: Serialize<E>,
{
    fn serialize_update(
        &self,
        stream: &mut Self::Stream,
        e: AnyEncoder<E>,
        mut ctx: Context,
    ) -> anyhow::Result<()> {
        let ref mut ids = *stream.subscriber.recv();
        let mut e = e.encode_seq(Some(ids.len()))?;
        for id in ids.drain() {
            self.map
                .get(&id)
                .unwrap()
                .serialize(e.encode_element()?, ctx.reborrow())?;
        }
        e.end()?;
        Ok(())
    }
}

impl<D: Decoder, C: Object> Deserialize<D> for ObjectMap<C>
where
    C::Pointer<C::Dyn>: DerefRaw<RawTarget = C::Dyn>,
    C::Dyn: RawAny,
    C::Pointer<C::Dyn>: Deserialize<D>,
{
    fn deserialize<'p, 'de>(d: AnyDecoder<'p, 'de, D>, ctx: Context) -> anyhow::Result<Self> {
        let mut result = Self::new();
        result.deserialize_update(d, ctx)?;
        Ok(result)
    }
}

impl<D: Decoder, C: Object> DeserializeUpdate<D> for ObjectMap<C>
where
    C::Pointer<C::Dyn>: DerefRaw<RawTarget = C::Dyn>,
    C::Dyn: RawAny,
    C::Pointer<C::Dyn>: Deserialize<D>,
{
    fn deserialize_update<'p, 'de>(
        &mut self,
        d: AnyDecoder<'p, 'de, D>,
        mut ctx: Context,
    ) -> anyhow::Result<()> {
        let mut d = d.decode(DecodeHint::Seq)?.try_into_seq()?;
        while let Some(d) = d.decode_next()? {
            self.insert(C::Pointer::<C::Dyn>::deserialize(d, ctx.reborrow())?);
        }
        Ok(())
    }
}
