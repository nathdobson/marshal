use std::any::Any;
use std::collections::{HashMap, HashSet};
use std::marker::Unsize;
use std::sync;
use std::sync::Arc;

use atomic_refcell::AtomicRefCell;
use by_address::ByAddress;
use tokenlock::{IcToken, IcTokenId, IcTokenLock};

use marshal::context::Context;
use marshal::de::rc::DeserializeArc;
use marshal::de::Deserialize;
use marshal::decode::{AnyDecoder, DecodeHint, Decoder};
use marshal::encode::{AnyEncoder, Encoder};
use marshal::ser::rc::SerializeArc;
use marshal::ser::Serialize;
use marshal_pointer::arc_ref::ArcRef;
use marshal_shared::de::deserialize_arc;
use marshal_shared::ser::SharedSerializeContext;

use crate::de::DeserializeUpdate;
use crate::ser::set_channel::{SetPublisher, SetSubscriber};
use crate::ser::{
    DeserializeUpdateDyn, SerializeStream, SerializeStreamDyn, SerializeUpdate, SerializeUpdateDyn,
};
use crate::tree::de::DynamicDecoder;
use crate::tree::ser::DynamicEncoder;
use crate::tree::TreeError;

mod address;

type ForestSharedSerializeContext = SharedSerializeContext<sync::Weak<Tree<dyn Sync + Send + Any>>>;
struct ForestSerializerTable {
    streamers:
        HashMap<ByAddress<Arc<Tree<dyn Sync + Send + Any>>>, Arc<Tree<dyn SerializeStreamDyn>>>,
    serializers: HashMap<ByAddress<Arc<Tree<dyn Sync + Send + Any>>>, Box<dyn Sync + Send + Any>>,
}
pub struct Forest {
    token: IcToken,
    publisher: SetPublisher<HashSet<ByAddress<Arc<Tree<dyn Sync + Send + Any>>>>>,
    serializers: AtomicRefCell<ForestSerializerTable>,
}

struct ForestDeserializerTable {
    token: IcTokenId,
    deserializers: HashMap<usize, Box<dyn Sync + Send + Any>>,
}

impl ForestDeserializerTable {
    fn new(token: IcTokenId) -> ForestDeserializerTable {
        ForestDeserializerTable {
            token,
            deserializers: HashMap::new(),
        }
    }
}

pub struct ForestRoot<T: ?Sized> {
    forest: Forest,
    deserializers: ForestDeserializerTable,
    root: T,
}

pub struct Tree<T: ?Sized> {
    inner: IcTokenLock<T>,
}

impl<T> ForestRoot<T> {
    pub fn new(forest: Forest, root: T) -> Self {
        let deserializers = ForestDeserializerTable::new(forest.token.id());
        ForestRoot {
            forest,
            deserializers,
            root,
        }
    }
    pub fn forest(&self) -> &Forest {
        &self.forest
    }
    pub fn forest_mut(&mut self) -> &mut Forest {
        &mut self.forest
    }
    pub fn root(&self) -> &T {
        &self.root
    }
    pub fn root_mut(&mut self) -> &mut T {
        &mut self.root
    }
}

impl Forest {
    pub fn new() -> Self {
        Forest {
            token: IcToken::new(),
            publisher: SetPublisher::new(),
            serializers: AtomicRefCell::new(ForestSerializerTable {
                streamers: HashMap::new(),
                serializers: HashMap::new(),
            }),
        }
    }
    pub fn add<T>(&self, value: T) -> Arc<Tree<T>> {
        Arc::new(self.add_raw(value))
    }
    pub fn add_raw<T>(&self, value: T) -> Tree<T> {
        Tree {
            inner: IcTokenLock::new(self.token.id(), value),
        }
    }
    pub fn get<'a, T: ?Sized>(&'a self, value: &'a Arc<Tree<T>>) -> &'a T {
        value.inner.read(&self.token)
    }
    pub fn get_mut<'a, T: 'static + ?Sized + Sync + Send>(
        &'a mut self,
        value: &'a Arc<Tree<T>>,
    ) -> &'a mut T
    where
        T: Unsize<dyn Sync + Send + Any>,
    {
        self.publisher.send(&ByAddress(value.clone()));
        value.inner.write(&mut self.token)
    }
}

impl<E: Encoder, T: Serialize<E>> Serialize<E> for ForestRoot<T> {
    fn serialize(&self, e: AnyEncoder<'_, E>, mut ctx: Context) -> anyhow::Result<()> {
        let mut ctx = ctx.clone_scoped();
        ctx.insert_const(&self.forest);
        let ref mut serializer_table = *self.forest.serializers.borrow_mut();
        ctx.insert_mut(serializer_table);
        let ctx = ctx.borrow();
        self.root.serialize(e, ctx)?;
        Ok(())
    }
}

pub struct ForestStream<T> {
    root: T,
    streams: HashMap<ByAddress<Arc<Tree<dyn Sync + Send + Any>>>, Box<dyn Sync + Send + Any>>,
    subscriber: SetSubscriber<HashSet<ByAddress<Arc<Tree<dyn Sync + Send + Any>>>>>,
}

impl<T: SerializeStream> SerializeStream for ForestRoot<T> {
    type Stream = ForestStream<T::Stream>;
    fn start_stream(&self, mut ctx: Context) -> anyhow::Result<Self::Stream> {
        let mut streams = HashMap::new();
        for (k, v) in &self.forest.serializers.borrow().streamers {
            streams.insert(
                k.clone(),
                self.forest.get(v).start_stream_dyn(ctx.reborrow())?,
            );
        }
        Ok(ForestStream {
            root: self.root.start_stream(ctx)?,
            streams,
            subscriber: self.forest.publisher.subscribe(),
        })
    }
}

impl<E: Encoder + DynamicEncoder, T: SerializeUpdate<E>> SerializeUpdate<E> for ForestRoot<T>
where
    E::SerializeUpdateDyn: Unsize<dyn Sync + Send + Any>,
    E::SerializeUpdateDyn: SerializeUpdateDyn<E>,
{
    fn serialize_update(
        &self,
        stream: &mut Self::Stream,
        e: AnyEncoder<E>,
        mut ctx: Context,
    ) -> anyhow::Result<()> {
        let mut ctx = ctx.clone_scoped();
        ctx.insert_const(&self.forest);
        let ref mut serializer_table = *self.forest.serializers.borrow_mut();
        ctx.insert_mut(serializer_table);
        let mut ctx = ctx.borrow();
        let mut e = e.encode_struct("ForestRoot", &["root", "trees"])?;
        self.root
            .serialize_update(&mut stream.root, e.encode_field()?, ctx.reborrow())?;
        {
            let ref mut addresses = *stream.subscriber.recv();
            let mut e = e.encode_field()?.encode_map(Some(addresses.len()))?;
            for address in addresses.drain() {
                let serializer = ctx
                    .reborrow()
                    .get_mut::<ForestSerializerTable>()?
                    .serializers
                    .get_mut(&address)
                    .ok_or(TreeError::MissingId)?
                    .downcast_ref::<Arc<Tree<E::SerializeUpdateDyn>>>()
                    .unwrap()
                    .clone();
                let serializer_ref = self.forest.get(&serializer);
                let stream = stream.streams.get_mut(&address).unwrap();
                let mut e = e.encode_entry()?;
                ForestSharedSerializeContext::get_id(
                    ctx.reborrow(),
                    Arc::downgrade(&serializer) as sync::Weak<Tree<dyn Sync + Send + Any>>,
                )?
                .unwrap()
                .serialize(e.encode_key()?, ctx.reborrow())?;
                serializer_ref.serialize_update_dyn(stream, e.encode_value()?, ctx.reborrow())?;

                e.end()?;
            }
            e.end()?;
        }
        e.end()?;
        Ok(())
    }
}

impl<E: Encoder + DynamicEncoder, T: 'static + Sync + Send + Serialize<E> + SerializeStream>
    SerializeArc<E> for Tree<T>
where
    T: Unsize<E::SerializeUpdateDyn>,
{
    fn serialize_arc(
        this: &ArcRef<Self>,
        e: AnyEncoder<'_, E>,
        mut ctx: Context,
    ) -> anyhow::Result<()> {
        let serializer_table = ctx.reborrow().get_mut::<ForestSerializerTable>()?;
        serializer_table
            .streamers
            .entry(ByAddress(this.arc()))
            .or_insert_with(|| this.arc() as Arc<Tree<dyn SerializeStreamDyn>>);
        serializer_table
            .serializers
            .entry(ByAddress(this.arc()))
            .or_insert_with(|| Box::new(this.arc() as Arc<Tree<E::SerializeUpdateDyn>>));
        ForestSharedSerializeContext::serialize_strong(&**this, this.weak(), e, ctx)?;
        Ok(())
    }
}

impl<E: Encoder, T: Serialize<E>> Serialize<E> for Tree<T> {
    fn serialize(&self, e: AnyEncoder<'_, E>, mut ctx: Context) -> anyhow::Result<()> {
        let (forest, ctx) = ctx.get_const_reborrow::<Forest>()?;
        self.inner.read(&forest.token).serialize(e, ctx)?;
        Ok(())
    }
}

impl<'de, D: Decoder<'de>, T: Deserialize<'de, D>> Deserialize<'de, D> for ForestRoot<T> {
    fn deserialize<'p>(d: AnyDecoder<'p, 'de, D>, mut ctx: Context) -> anyhow::Result<Self> {
        let mut forest = Forest::new();
        let mut deserializers = ForestDeserializerTable::new(forest.token.id());
        let mut ctx = ctx.insert_mut_scoped(&mut deserializers);
        let root = T::deserialize(d, ctx.borrow())?;
        Ok(ForestRoot {
            forest,
            deserializers,
            root,
        })
    }
}

impl<'de, D: Decoder<'de> + DynamicDecoder, T: DeserializeUpdate<'de, D>> DeserializeUpdate<'de, D>
    for ForestRoot<T>
where
    D::DeserializeUpdateDyn: DeserializeUpdateDyn<'de, D>,
{
    fn deserialize_update<'p>(
        &mut self,
        d: AnyDecoder<'p, 'de, D>,
        mut ctx: Context,
    ) -> anyhow::Result<()> {
        let mut ctx = ctx.clone_scoped();
        ctx.insert_mut::<ForestDeserializerTable>(&mut self.deserializers);
        let mut ctx = ctx.borrow();
        let mut d = d.decode_struct_helper("ForestRoot", &["root", "trees"])?;
        while let Some((field, mut d)) = d.next()? {
            match field {
                0 => self
                    .root
                    .deserialize_update(d.decode_field()?, ctx.reborrow())?,
                1 => {
                    let mut d = d.decode_field()?.decode(DecodeHint::Map)?.try_into_map()?;
                    while let Some(mut d) = d.decode_next()? {
                        let key = usize::deserialize(d.decode_key()?, ctx.reborrow())?;
                        let des: Arc<Tree<D::DeserializeUpdateDyn>> = (**ctx
                            .reborrow()
                            .get_mut::<ForestDeserializerTable>()?
                            .deserializers
                            .get(&key)
                            .ok_or(TreeError::MissingId)?)
                        .downcast_ref::<Arc<Tree<D::DeserializeUpdateDyn>>>()
                        .unwrap()
                        .clone();
                        self.forest
                            .get_mut(&des)
                            .deserialize_update_dyn(d.decode_value()?, ctx.reborrow())?;
                        d.decode_end()?;
                    }
                }
                _ => unreachable!(),
            }
            d.decode_end()?;
        }
        Ok(())
    }
}

impl<
        'de,
        D: Decoder<'de> + DynamicDecoder,
        T: 'static + Sync + Send + DeserializeUpdate<'de, D>,
    > DeserializeArc<'de, D> for Tree<T>
where
    T: Unsize<D::DeserializeUpdateDyn>,
{
    fn deserialize_arc<'p>(
        p: AnyDecoder<'p, 'de, D>,
        mut ctx: Context,
    ) -> anyhow::Result<Arc<Self>> {
        let (id, arc) = deserialize_arc::<D, Tree<T>>(p, ctx.reborrow())?;
        ctx.get_mut::<ForestDeserializerTable>()?
            .deserializers
            .insert(
                id,
                Box::new(arc.clone() as Arc<Tree<D::DeserializeUpdateDyn>>),
            );
        Ok(arc)
    }
}

impl<'de, D: Decoder<'de>, T: Deserialize<'de, D>> Deserialize<'de, D> for Tree<T> {
    fn deserialize<'p>(d: AnyDecoder<'p, 'de, D>, mut ctx: Context) -> anyhow::Result<Self> {
        let tree = T::deserialize(d, ctx.reborrow())?;
        Ok(Tree {
            inner: IcTokenLock::new(ctx.get_mut::<ForestDeserializerTable>()?.token, tree),
        })
    }
}
