use crate::tree::Tree;
use marshal::context::Context;
use marshal::de::rc::DeserializeArc;
use marshal::de::Deserialize;
use marshal::decode::{AnyDecoder, DecodeHint, Decoder};
use marshal_shared::de::{deserialize_arc, SharedArcDeserializeContext};
use std::any::Any;
use std::collections::HashMap;
use std::marker::{PhantomData, Unsize};
use std::sync::Arc;

pub struct DeserializeForest<S: ?Sized> {
    phantom: PhantomData<S>,
    entries: HashMap<usize, Arc<Tree<S>>>,
}

impl<'de, D: Decoder<'de>, T: Sync + Send + Deserialize<'de, D>> Deserialize<'de, D> for Tree<T> {
    fn deserialize<'p>(d: AnyDecoder<'p, 'de, D>, ctx: &mut Context) -> anyhow::Result<Self> {
        Ok(Tree::new(T::deserialize(d, ctx)?))
    }
}

impl<'de, D: Decoder<'de> + DynamicDecoder, T: 'static + Sync + Send + Deserialize<'de, D>>
    DeserializeArc<'de, D> for Tree<T>
where
    T: Unsize<D::DeserializeUpdateDyn>,
{
    fn deserialize_arc<'p>(
        d: AnyDecoder<'p, 'de, D>,
        ctx: &mut Context,
    ) -> anyhow::Result<Arc<Self>> {
        let (id, result) = deserialize_arc::<D, Tree<T>>(d, ctx)?;
        let forest = ctx.get_mut::<DeserializeForest<D::DeserializeUpdateDyn>>()?;
        forest.entries.insert(id, result.clone());
        Ok(result)
    }
}

pub trait DynamicDecoder {
    type DeserializeUpdateDyn: 'static + ?Sized;
}

impl<S: ?Sized> DeserializeForest<S> {
    pub fn new() -> Self {
        DeserializeForest {
            phantom: PhantomData,
            entries: HashMap::new(),
        }
    }
    pub fn deserialize_updates<'de, D: Decoder<'de>>(
        d: AnyDecoder<'_, 'de, D>,
        ctx: &mut Context,
    ) -> anyhow::Result<()> {
        let mut d = d.decode(DecodeHint::Map)?.try_into_map()?;
        while let Some(mut d) = d.decode_next()? {
            let id = usize::deserialize(d.decode_key()?, ctx)?;
        }
        Ok(())
    }
}
