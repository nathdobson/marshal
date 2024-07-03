use std::any::Any;
use std::marker::{PhantomData, Unsize};
use std::ops::{CoerceUnsized, DerefMut};

use marshal::context::Context;
use marshal::de::Deserialize;
use marshal::decode::{AnyDecoder, Decoder};
use marshal::encode::{AnyEncoder, Encoder};
use marshal::ser::Serialize;
use marshal_object::Object;
use marshal_object::type_id::ObjectTypeId;
use marshal_pointer::{AsFlatRef, DerefRaw, DowncastRef, RawAny};

use crate::de::DeserializeUpdate;
use crate::hash_map;
use crate::hash_map::{UpdateHashMap, UpdateHashMapStream};
use crate::ser::{SerializeStream, SerializeUpdate};

pub struct ObjectMap<C: Object> {
    map: UpdateHashMap<ObjectTypeId<C>, C::Pointer<C::Dyn>>,
}

pub struct ObjectMapStream<C: Object, S> {
    stream: UpdateHashMapStream<ObjectTypeId<C>, S>,
}

pub enum Entry<'a, C: Object, T> {
    Occupied(OccupiedEntry<'a, C, T>),
    Vacant(VacantEntry<'a, C, T>),
}

pub struct OccupiedEntry<'a, C: Object, T> {
    inner: hash_map::OccupiedEntry<'a, ObjectTypeId<C>, C::Pointer<C::Dyn>>,
    phantom: PhantomData<&'a T>,
}

pub struct VacantEntry<'a, C: Object, T> {
    inner: hash_map::VacantEntry<'a, ObjectTypeId<C>, C::Pointer<C::Dyn>>,
    phantom: PhantomData<&'a T>,
}

impl<C: Object> ObjectMap<C> {
    pub fn new() -> Self {
        ObjectMap {
            map: UpdateHashMap::new(),
        }
    }
    pub fn get<T: 'static>(&self) -> Option<&<C::Pointer<T> as AsFlatRef>::FlatRef>
    where
        for<'a> &'a <C::Pointer<C::Dyn> as AsFlatRef>::FlatRef:
            CoerceUnsized<&'a <C::Pointer<dyn RawAny> as AsFlatRef>::FlatRef>,
        <C::Pointer<dyn RawAny> as AsFlatRef>::FlatRef:
            DowncastRef<<C::Pointer<T> as AsFlatRef>::FlatRef>,
        T: Unsize<C::Dyn>,
    {
        let dyn_flat_ref: &<C::Pointer<C::Dyn> as AsFlatRef>::FlatRef =
            self.map.get(&ObjectTypeId::of::<T>())?.as_flat_ref();
        let any_flat_ref: &<C::Pointer<dyn RawAny> as AsFlatRef>::FlatRef = dyn_flat_ref;
        let flat_ref: &<C::Pointer<T> as AsFlatRef>::FlatRef = any_flat_ref.downcast_ref().unwrap();
        Some(flat_ref)
    }
    pub fn get_mut<T: 'static>(&mut self) -> Option<&mut T>
    where
        C::Pointer<C::Dyn>: DerefMut<Target = C::Dyn>,
        T: Unsize<C::Dyn>,
        C::Dyn: Unsize<dyn Any>,
    {
        Some(
            (self.map.get_mut(&ObjectTypeId::of::<T>())?.deref_mut() as &mut dyn Any)
                .downcast_mut::<T>()
                .unwrap(),
        )
    }
    // pub fn get_or_default<T: 'static>(&mut self) -> &<C::Pointer<T> as AsFlatRef>::FlatRef
    // where
    //     C::Pointer<T>: Default,
    //     C::Pointer<T>: CoerceUnsized<C::Pointer<C::Dyn>>,
    //     <C::Pointer<C::Dyn> as AsFlatRef>::FlatRef:
    //         Unsize<<C::Pointer<dyn RawAny> as AsFlatRef>::FlatRef>,
    //     <C::Pointer<dyn RawAny> as AsFlatRef>::FlatRef:
    //         DowncastRef<<C::Pointer<T> as AsFlatRef>::FlatRef>,
    // {
    //     (self
    //         .map
    //         .entry(TypeId::of::<T>())
    //         .or_insert_with(|| C::Pointer::<T>::default())
    //         .as_flat_ref() as &<C::Pointer<dyn RawAny> as AsFlatRef>::FlatRef)
    //         .downcast_ref()
    //         .unwrap()
    // }
    // pub fn get_or_insert_with<T: 'static, F>(
    //     &mut self,
    //     f: F,
    // ) -> &<C::Pointer<T> as AsFlatRef>::FlatRef
    // where
    //     C::Pointer<T>: Default,
    //     C::Pointer<T>: CoerceUnsized<C::Pointer<C::Dyn>>,
    //     <C::Pointer<C::Dyn> as AsFlatRef>::FlatRef:
    //         Unsize<<C::Pointer<dyn RawAny> as AsFlatRef>::FlatRef>,
    //     <C::Pointer<dyn RawAny> as AsFlatRef>::FlatRef:
    //         DowncastRef<<C::Pointer<T> as AsFlatRef>::FlatRef>,
    //     F: FnOnce() -> C::Pointer<T>,
    // {
    //     (self
    //         .map
    //         .entry(TypeId::of::<T>())
    //         .or_insert_with(|| f())
    //         .as_flat_ref() as &<C::Pointer<dyn RawAny> as AsFlatRef>::FlatRef)
    //         .downcast_ref()
    //         .unwrap()
    // }
    pub fn insert(&mut self, value: C::Pointer<C::Dyn>)
    where
        C::Pointer<C::Dyn>: DerefRaw<RawTarget = C::Dyn>,
        C::Dyn: RawAny,
    {
        self.map
            .insert(ObjectTypeId::of_dyn(value.deref_raw()), value);
    }
    pub fn entry<'a, T>(&'a mut self) -> Entry<'a, C, T>
    where
        T: Unsize<C::Dyn>,
    {
        match self.map.entry(ObjectTypeId::of::<T>()) {
            hash_map::Entry::Occupied(o) => Entry::Occupied(OccupiedEntry {
                inner: o,
                phantom: PhantomData,
            }),
            hash_map::Entry::Vacant(v) => Entry::Vacant(VacantEntry {
                inner: v,
                phantom: PhantomData,
            }),
        }
    }
}

impl<E: Encoder, C: Object> Serialize<E> for ObjectMap<C>
where
    C::Pointer<C::Dyn>: Serialize<E>,
{
    fn serialize<'w, 'en>(&self, e: AnyEncoder<'w, 'en, E>, ctx: Context) -> anyhow::Result<()> {
        self.map.serialize(e, ctx)
    }
}

impl<C: Object> SerializeStream for ObjectMap<C>
where
    C::Pointer<C::Dyn>: SerializeStream,
{
    type Stream = ObjectMapStream<C, <C::Pointer<C::Dyn> as SerializeStream>::Stream>;
    fn start_stream(&self, ctx: Context) -> anyhow::Result<Self::Stream> {
        Ok(ObjectMapStream {
            stream: self.map.start_stream(ctx)?,
        })
    }
}

impl<E: Encoder, C: Object> SerializeUpdate<E> for ObjectMap<C>
where
    C::Pointer<C::Dyn>: SerializeUpdate<E>,
{
    fn serialize_update(
        &self,
        stream: &mut Self::Stream,
        e: AnyEncoder<E>,
        ctx: Context,
    ) -> anyhow::Result<()> {
        self.map.serialize_update(&mut stream.stream, e, ctx)
    }
}

impl<D: Decoder, C: Object> Deserialize<D> for ObjectMap<C>
where
    C::Pointer<C::Dyn>: Deserialize<D>,
{
    fn deserialize<'p, 'de>(d: AnyDecoder<'p, 'de, D>, ctx: Context) -> anyhow::Result<Self> {
        Ok(ObjectMap {
            map: UpdateHashMap::deserialize(d, ctx)?,
        })
    }
}

impl<D: Decoder, C: Object> DeserializeUpdate<D> for ObjectMap<C>
where
    C::Pointer<C::Dyn>: DeserializeUpdate<D>,
{
    fn deserialize_update<'p, 'de>(
        &mut self,
        d: AnyDecoder<'p, 'de, D>,
        ctx: Context,
    ) -> anyhow::Result<()> {
        self.map.deserialize_update(d, ctx)
    }
}
impl<'a, C: Object, T: 'static> Entry<'a, C, T> {
    pub fn or_default(self) -> &'a <C::Pointer<T> as AsFlatRef>::FlatRef
    where
        C::Pointer<T>: CoerceUnsized<C::Pointer<C::Dyn>>,
        <C::Pointer<C::Dyn> as AsFlatRef>::FlatRef:
            Unsize<<C::Pointer<dyn RawAny> as AsFlatRef>::FlatRef>,
        <C::Pointer<dyn RawAny> as AsFlatRef>::FlatRef:
            DowncastRef<<C::Pointer<T> as AsFlatRef>::FlatRef>,
        C::Pointer<C::Dyn>: DerefMut<Target = C::Dyn>,
        C::Pointer<T>: Default,
    {
        self.or_insert_with(C::Pointer::<T>::default)
    }
    pub fn or_insert_with<F: FnOnce() -> C::Pointer<T>>(
        self,
        f: F,
    ) -> &'a <C::Pointer<T> as AsFlatRef>::FlatRef
    where
        C::Pointer<T>: CoerceUnsized<C::Pointer<C::Dyn>>,
        <C::Pointer<C::Dyn> as AsFlatRef>::FlatRef:
            Unsize<<C::Pointer<dyn RawAny> as AsFlatRef>::FlatRef>,
        <C::Pointer<dyn RawAny> as AsFlatRef>::FlatRef:
            DowncastRef<<C::Pointer<T> as AsFlatRef>::FlatRef>,
        C::Pointer<C::Dyn>: DerefMut<Target = C::Dyn>,
    {
        match self {
            Entry::Occupied(o) => o.into_ref(),
            Entry::Vacant(v) => v.insert(f()),
        }
    }
    pub fn or_insert_with_mut<F: FnOnce() -> C::Pointer<T>>(self, f: F) -> &'a mut T
    where
        C::Pointer<T>: CoerceUnsized<C::Pointer<C::Dyn>>,
        C::Pointer<C::Dyn>: DerefMut<Target = C::Dyn>,
        C::Dyn: Unsize<dyn Any>,
    {
        match self {
            Entry::Occupied(o) => o.into_mut(),
            Entry::Vacant(v) => v.insert_mut(f()),
        }
    }
    pub fn or_default_mut(self) -> &'a mut T
    where
        C::Pointer<T>: CoerceUnsized<C::Pointer<C::Dyn>>,
        C::Pointer<C::Dyn>: DerefMut<Target = C::Dyn>,
        C::Dyn: Unsize<dyn Any>,
        C::Pointer<T>: Default,
    {
        self.or_insert_with_mut(C::Pointer::<T>::default)
    }
}

impl<'a, C: Object, T: 'static> VacantEntry<'a, C, T> {
    pub fn insert(self, value: C::Pointer<T>) -> &'a <C::Pointer<T> as AsFlatRef>::FlatRef
    where
        C::Pointer<T>: CoerceUnsized<C::Pointer<C::Dyn>>,
        <C::Pointer<C::Dyn> as AsFlatRef>::FlatRef:
            Unsize<<C::Pointer<dyn RawAny> as AsFlatRef>::FlatRef>,
        <C::Pointer<dyn RawAny> as AsFlatRef>::FlatRef:
            DowncastRef<<C::Pointer<T> as AsFlatRef>::FlatRef>,
    {
        (self.inner.insert(value).as_flat_ref() as &<C::Pointer<C::Dyn> as AsFlatRef>::FlatRef
            as &<C::Pointer<dyn RawAny> as AsFlatRef>::FlatRef)
            .downcast_ref()
            .unwrap()
    }
    pub fn insert_mut(self, value: C::Pointer<T>) -> &'a mut T
    where
        C::Pointer<T>: CoerceUnsized<C::Pointer<C::Dyn>>,
        C::Pointer<C::Dyn>: DerefMut<Target = C::Dyn>,
        C::Dyn: Unsize<dyn Any>,
    {
        (self.inner.insert(value).deref_mut() as &mut dyn Any)
            .downcast_mut()
            .unwrap()
    }
}

impl<'a, C: Object, T: 'static> OccupiedEntry<'a, C, T> {
    pub fn into_ref(self) -> &'a <C::Pointer<T> as AsFlatRef>::FlatRef
    where
        <C::Pointer<C::Dyn> as AsFlatRef>::FlatRef:
            Unsize<<C::Pointer<dyn RawAny> as AsFlatRef>::FlatRef>,
        <C::Pointer<dyn RawAny> as AsFlatRef>::FlatRef:
            DowncastRef<<C::Pointer<T> as AsFlatRef>::FlatRef>,
    {
        (self.inner.into_mut().as_flat_ref() as &<C::Pointer<dyn RawAny> as AsFlatRef>::FlatRef)
            .downcast_ref()
            .unwrap()
    }
    pub fn into_mut(self) -> &'a mut T
    where
        C::Pointer<C::Dyn>: DerefMut<Target = C::Dyn>,
        C::Dyn: Unsize<dyn Any>,
    {
        (self.inner.into_mut().deref_mut() as &mut dyn Any)
            .downcast_mut()
            .unwrap()
    }
}

impl<C: Object> Default for ObjectMap<C> {
    fn default() -> Self {
        ObjectMap::new()
    }
}
