use std::any::Any;
use std::collections::{HashMap, HashSet};
use std::hash::Hash;
use std::marker::Unsize;
use std::sync::Arc;
use std::{mem, sync};

use atomic_refcell::AtomicRefCell;
use by_address::ByThinAddress;
use parking_lot::Mutex;

use marshal::context::Context;
use marshal::encode::{AnyEncoder, Encoder};
use marshal::ser::rc::SerializeArc;
use marshal::ser::Serialize;
use marshal_json::encode::full::JsonEncoder;
use marshal_pointer::arc_ref::ArcRef;
use marshal_pointer::DerefRaw;
use marshal_shared::ser::SharedSerializeContext;

use crate::ser::SerializeUpdateDyn;
use crate::tree::json::SerializeUpdateJson;
use crate::tree::{Tree, TreeError};

#[derive(Eq, Ord, PartialEq, PartialOrd, Hash)]
pub(crate) struct Address(*const ());

impl<'a, T: ?Sized> From<&'a ArcRef<T>> for Address {
    fn from(value: &'a ArcRef<T>) -> Self {
        Address(value.deref_raw() as *const T as *const ())
    }
}

pub struct SerializeForest<S: ?Sized> {
    pub(crate) queue: Arc<SerializeQueue>,
    pub(crate) serializers: HashMap<Address, Arc<Tree<S>>>,
}

pub struct SerializeQueue {
    pub(crate) queue: Mutex<HashSet<ByThinAddress<Arc<Tree<dyn Any>>>>>,
}

impl<E: Encoder, T: Serialize<E>> Serialize<E> for Tree<T> {
    fn serialize(&self, e: AnyEncoder<'_, E>, ctx: &mut Context) -> anyhow::Result<()> {
        self.state.borrow().value.serialize(e, ctx)
    }
}

pub trait DynamicEncoder {
    type SerializeUpdateDyn: 'static + ?Sized;
}

impl<E, T> SerializeArc<E> for Tree<T>
where
    E: Encoder + DynamicEncoder,
    T: 'static + Unsize<E::SerializeUpdateDyn> + Serialize<E> + SerializeUpdateDyn<E>,
{
    fn serialize_arc(
        this: &ArcRef<Self>,
        e: AnyEncoder<'_, E>,
        ctx: &mut Context,
    ) -> anyhow::Result<()> {
        let forest = ctx.get_mut::<SerializeForest<E::SerializeUpdateDyn>>()?;
        forest.serializers.insert(this.into(), this.arc());
        this.forest.get_or_init(|| forest.queue.clone());
        {
            let ref mut state = *this.state.borrow_mut();
            let stream = &mut state.stream;
            let value = &mut state.value;
            state.stream = Some(value.start_stream_dyn(ctx)?);
        }
        SharedSerializeContext::<sync::Weak<Tree<dyn Any>>>::serialize_strong(
            &**this,
            this.weak(),
            e,
            ctx,
        )?;
        Ok(())
    }
}

impl<S: ?Sized> SerializeForest<S> {
    pub fn new() -> Self {
        SerializeForest {
            queue: Arc::new(SerializeQueue {
                queue: Mutex::new(HashSet::new()),
            }),
            serializers: HashMap::new(),
        }
    }
    pub fn queue(&self) -> &Arc<SerializeQueue> {
        &self.queue
    }
    pub fn serialize_updates<E: Encoder>(e: AnyEncoder<E>, ctx: &mut Context) -> anyhow::Result<()>
    where
        S: SerializeUpdateDyn<E>,
    {
        let queue = mem::replace(
            &mut *ctx.get_mut::<Self>()?.queue.queue.lock(),
            HashSet::new(),
        );
        let mut e = e.encode_map(None)?;
        for tree in queue {
            let tree = tree.0;
            let serializer = ctx
                .get_mut::<Self>()?
                .serializers
                .get(&Address(&*tree as *const Tree<dyn Any> as *const ()))
                .unwrap()
                .clone();
            let ref mut state = *serializer.state.borrow_mut();
            let value = &mut state.value;
            let stream = state.stream.as_mut().unwrap();
            let mut e = e.encode_entry()?;
            let id = SharedSerializeContext::<sync::Weak<Tree<dyn Any>>>::get_id(
                ctx,
                Arc::downgrade(&tree),
            )
            .ok_or(TreeError::MissingId)?;
            id.serialize(e.encode_key()?, ctx)?;
            value.serialize_update_dyn(&mut **stream, e.encode_value()?, ctx)?;
            e.end()?;
        }
        e.end()?;
        Ok(())
    }
}
