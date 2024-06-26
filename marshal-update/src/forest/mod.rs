mod address;

use crate::de::DeserializeUpdate;
use crate::forest::address::Address;
use crate::ser::{SerializeStream, SerializeUpdate};
use marshal::context::Context;
use marshal::de::rc::DeserializeArc;
use marshal::de::Deserialize;
use marshal::decode::{AnyDecoder, Decoder};
use marshal::encode::{AnyEncoder, Encoder};
use marshal::ser::rc::SerializeArc;
use marshal::ser::Serialize;
use marshal_pointer::arc_ref::ArcRef;
use marshal_shared::ser::serialize_arc;
use std::any::Any;
use std::collections::HashMap;
use std::sync::Arc;
use tokenlock::{IcToken, IcTokenLock};

pub struct Forest {
    token: IcToken,
}

pub struct ForestRoot<T: ?Sized> {
    forest: Forest,
    root: T,
}

pub struct Tree<T: ?Sized> {
    inner: IcTokenLock<T>,
}

impl<T> ForestRoot<T> {
    pub fn new(forest: Forest, root: T) -> Self {
        ForestRoot { forest, root }
    }
}

impl Forest {
    pub fn new() -> Self {
        Forest {
            token: IcToken::new(),
        }
    }
    pub fn add<T>(&self, value: T) -> Arc<Tree<T>> {
        Arc::new(Tree {
            inner: IcTokenLock::new(self.token.id(), value),
        })
    }
    pub fn get<'a, T>(&'a self, value: &'a Arc<Tree<T>>) -> &'a T {
        value.inner.read(&self.token)
    }
    pub fn get_mut<'a, T>(&'a mut self, value: &'a Arc<Tree<T>>) -> &'a mut T {
        value.inner.write(&mut self.token)
    }
}

impl<E: Encoder, T: Serialize<E>> Serialize<E> for ForestRoot<T> {
    fn serialize(&self, e: AnyEncoder<'_, E>, mut ctx: Context) -> anyhow::Result<()> {
        let mut ctx = ctx.insert_const_scoped(&self.forest);
        self.root.serialize(e, ctx.borrow())?;
        Ok(())
    }
}

pub struct ForestStream<T> {
    root: T,
    streams: HashMap<Address, Arc<Tree<dyn Sync + Send + Any>>>,
}

impl<T: SerializeStream> SerializeStream for ForestRoot<T> {
    type Stream = ForestStream<T::Stream>;
    fn start_stream(&self, ctx: Context) -> anyhow::Result<Self::Stream> {
        Ok(ForestStream {
            root: self.root.start_stream(ctx)?,
            streams: HashMap::new(),
        })
    }
}

impl<E: Encoder, T: SerializeUpdate<E>> SerializeUpdate<E> for ForestRoot<T> {
    fn serialize_update(
        &self,
        stream: &mut Self::Stream,
        e: AnyEncoder<E>,
        ctx: Context,
    ) -> anyhow::Result<()> {
        todo!()
    }
}

impl<E: Encoder, T: 'static + Serialize<E>> SerializeArc<E> for Tree<T> {
    fn serialize_arc(
        this: &ArcRef<Self>,
        e: AnyEncoder<'_, E>,
        ctx: Context,
    ) -> anyhow::Result<()> {
        serialize_arc(this, e, ctx)?;
        Ok(())
    }
}

impl<E: Encoder, T: Serialize<E>> Serialize<E> for Tree<T> {
    fn serialize(&self, e: AnyEncoder<'_, E>, ctx: Context) -> anyhow::Result<()> {
        let forest = ctx.get_const::<Forest>()?;
        todo!();
    }
}

impl<'de, D: Decoder<'de>, T: Deserialize<'de, D>> Deserialize<'de, D> for ForestRoot<T> {
    fn deserialize<'p>(d: AnyDecoder<'p, 'de, D>, ctx: Context) -> anyhow::Result<Self> {
        todo!()
    }
}

impl<'de, D: Decoder<'de>, T: DeserializeUpdate<'de, D>> DeserializeUpdate<'de, D>
    for ForestRoot<T>
{
    fn deserialize_update<'p>(
        &mut self,
        d: AnyDecoder<'p, 'de, D>,
        ctx: Context,
    ) -> anyhow::Result<()> {
        todo!()
    }
}

impl<'de, D: Decoder<'de>, T: DeserializeUpdate<'de, D>> DeserializeArc<'de, D> for Tree<T> {
    fn deserialize_arc<'p>(p: AnyDecoder<'p, 'de, D>, ctx: Context) -> anyhow::Result<Arc<Self>> {
        todo!()
    }
}
