use std::any::Any;
use std::collections::{HashMap, HashSet};
use std::sync;
use std::sync::Arc;

use by_address::ByAddress;

use marshal::context::Context;
use marshal::encode::{AnyGenEncoder,  GenEncoder};
use marshal::ser::rc::SerializeArc;
use marshal::ser::Serialize;
use marshal_pointer::arc_ref::ArcRef;
use marshal_shared::ser::SharedSerializeContext;

use crate::forest::error::TreeError;
use crate::forest::forest::{Forest, ForestRoot, Tree};
use crate::ser::{SerializeStream, SerializeStreamDyn, SerializeUpdate, SerializeUpdateDyn};
use crate::ser::set_channel::SetSubscriber;

type ForestSharedSerializeContext = SharedSerializeContext<sync::Weak<Tree<dyn Sync + Send + Any>>>;
pub(super) struct ForestSerializerTable {
    streamers:
        HashMap<ByAddress<Arc<Tree<dyn Sync + Send + Any>>>, Arc<Tree<dyn SerializeStreamDyn>>>,
    serializers: HashMap<ByAddress<Arc<Tree<dyn Sync + Send + Any>>>, Box<dyn Sync + Send + Any>>,
}

impl ForestSerializerTable {
    pub fn new() -> Self {
        ForestSerializerTable {
            streamers: HashMap::new(),
            serializers: HashMap::new(),
        }
    }
}

impl<E: GenEncoder, T: Serialize<E>> Serialize<E> for ForestRoot<T> {
    fn serialize<'w, 'en>(
        &self,
        e: AnyGenEncoder<'w, 'en, E>,
        mut ctx: Context,
    ) -> anyhow::Result<()> {
        let forest = self.forest();
        let root = self.root();
        let ref mut serializer_table = *forest.serializers().borrow_mut();

        let mut ctx = ctx.clone_scoped();
        ctx.insert_const(forest);
        ctx.insert_mut(serializer_table);
        let ctx = ctx.borrow();

        root.serialize(e, ctx)?;
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
        for (k, v) in &self.forest().serializers().borrow().streamers {
            streams.insert(
                k.clone(),
                self.forest().get(v).start_stream_dyn(ctx.reborrow())?,
            );
        }
        Ok(ForestStream {
            root: self.root().start_stream(ctx)?,
            streams,
            subscriber: self.forest().subscribe(),
        })
    }
}

impl<E: GenEncoder, T: SerializeUpdate<E>> SerializeUpdate<E> for ForestRoot<T> {
    fn serialize_update(
        &self,
        stream: &mut Self::Stream,
        e: AnyGenEncoder<E>,
        mut ctx: Context,
    ) -> anyhow::Result<()> {
        let forest = self.forest();
        let ref mut serializer_table = *forest.serializers().borrow_mut();

        let mut ctx = ctx.clone_scoped();
        ctx.insert_const(forest);
        ctx.insert_mut(serializer_table);
        let mut ctx = ctx.borrow();

        let mut e = e.encode_struct("ForestRoot", &["root", "trees"])?;
        self.root()
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
                    .downcast_ref::<Arc<Tree<dyn Sync + Send + SerializeUpdateDyn<E>>>>()
                    .unwrap()
                    .clone();
                let serializer_ref = self.forest().get(&serializer);
                let stream = stream.streams.get_mut(&address).unwrap();
                let mut e = e.encode_entry()?;
                let id = ForestSharedSerializeContext::get_id(
                    ctx.reborrow(),
                    Arc::downgrade(&serializer) as sync::Weak<Tree<dyn Sync + Send + Any>>,
                )?
                .unwrap();
                <usize as Serialize<E>>::serialize(&id, e.encode_key()?, ctx.reborrow())?;
                serializer_ref.serialize_update_dyn(stream, e.encode_value()?, ctx.reborrow())?;

                e.end()?;
            }
            e.end()?;
        }
        e.end()?;
        Ok(())
    }
}

impl<
        E: GenEncoder,
        T: 'static + Sync + Send + Serialize<E> + SerializeStream + SerializeUpdate<E>,
    > SerializeArc<E> for Tree<T>
{
    fn serialize_arc<'w, 'en>(
        this: &ArcRef<Self>,
        e: AnyGenEncoder<'w, 'en, E>,
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
            .or_insert_with(|| {
                Box::new(this.arc() as Arc<Tree<dyn Sync + Send + SerializeUpdateDyn<E>>>)
            });
        ForestSharedSerializeContext::serialize_strong(&**this, this.weak(), e, ctx)?;
        Ok(())
    }
}

impl<E: GenEncoder, T: Serialize<E>> Serialize<E> for Tree<T> {
    fn serialize<'w, 'en>(
        &self,
        e: AnyGenEncoder<'w, 'en, E>,
        mut ctx: Context,
    ) -> anyhow::Result<()> {
        let (forest, ctx) = ctx.get_const_reborrow::<Forest>()?;
        forest.get_raw(&self).serialize(e, ctx)?;
        Ok(())
    }
}
