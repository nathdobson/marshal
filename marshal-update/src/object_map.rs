use std::any::TypeId;
use std::collections::{HashMap, HashSet};
use std::ops::CoerceUnsized;

use marshal::context::Context;
use marshal::encode::{AnyEncoder, Encoder};
use marshal::ser::Serialize;
use marshal_object::Object;
use marshal_pointer::{AsFlatRef, DerefRaw, DowncastRef, RawAny};

use crate::ser::{SerializeStream, SerializeUpdate};
use crate::ser::set_channel::{SetPublisher, SetSubscriber};

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
    pub fn get<T: 'static>(&self) -> Option<&T>
    where
        <C::Pointer<C::Dyn> as AsFlatRef>::FlatRef: DowncastRef<T>,
    {
        Some(
            self.map
                .get(&TypeId::of::<T>())?
                .as_flat_ref()
                .downcast_ref()
                .unwrap(),
        )
    }
    pub fn get_or_default<T: 'static>(&mut self) -> &T
    where
        C::Pointer<T>: Default,
        C::Pointer<T>: CoerceUnsized<C::Pointer<C::Dyn>>,
        <C::Pointer<C::Dyn> as AsFlatRef>::FlatRef: DowncastRef<T>,
    {
        self.map
            .entry(TypeId::of::<T>())
            .or_insert_with(|| C::Pointer::<T>::default())
            .as_flat_ref()
            .downcast_ref()
            .unwrap()
    }
    pub fn insert(&mut self, value: C::Pointer<C::Dyn>)
    where
        C::Pointer<C::Dyn>: DerefRaw<RawTarget = C::Dyn>,
        C::Dyn: RawAny,
    {
        self.map.insert(value.deref_raw().raw_type_id(), value);
    }
}

impl<E: Encoder, C: Object> Serialize<E> for ObjectMap<C>
where
    C::Pointer<C::Dyn>: Serialize<E>,
{
    fn serialize(&self, e: AnyEncoder<'_, E>, ctx: &mut Context) -> anyhow::Result<()> {
        let mut e = e.encode_seq(Some(self.map.len()))?;
        for x in self.map.values() {
            x.serialize(e.encode_element()?, ctx)?;
        }
        e.end()?;
        Ok(())
    }
}

impl<C: Object> SerializeStream for ObjectMap<C> {
    type Stream = ObjectMapStream;

    fn start_stream(&self, ctx: &mut Context) -> anyhow::Result<Self::Stream> {
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
        ctx: &mut Context,
    ) -> anyhow::Result<()> {
        let ref mut ids = *stream.subscriber.recv();
        let mut e = e.encode_seq(Some(ids.len()))?;
        for id in ids.drain() {
            self.map
                .get(&id)
                .unwrap()
                .serialize(e.encode_element()?, ctx)?;
        }
        e.end()?;
        Ok(())
    }
}
