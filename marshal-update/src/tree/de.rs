use std::collections::HashMap;
use std::marker::{PhantomData, Unsize};
use std::sync::{Arc, Weak};

use marshal::context::Context;
use marshal::de::Deserialize;
use marshal::de::rc::{DeserializeArc, DeserializeArcWeak};
use marshal::decode::{AnyDecoder, DecodeHint, Decoder};
use marshal_shared::de::{deserialize_arc, deserialize_arc_weak};

use crate::ser::DeserializeUpdateDyn;
use crate::tree::{Tree, TreeError};

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

impl<'de, D: Decoder<'de> + DynamicDecoder, T: 'static + Sync + Send + Deserialize<'de, D>>
    DeserializeArcWeak<'de, D> for Tree<T>
{
    fn deserialize_arc_weak<'p>(
        d: AnyDecoder<'p, 'de, D>,
        ctx: &mut Context,
    ) -> anyhow::Result<Weak<Self>> {
        let (_, result) = deserialize_arc_weak::<D, Tree<T>>(d, ctx)?;
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
    ) -> anyhow::Result<()>
    where
        S: DeserializeUpdateDyn<'de, D>,
    {
        let mut d = d.decode(DecodeHint::Map)?.try_into_map()?;
        while let Some(mut d) = d.decode_next()? {
            let id = usize::deserialize(d.decode_key()?, ctx)?;
            let value = ctx
                .get_mut::<DeserializeForest<S>>()?
                .entries
                .get(&id)
                .ok_or(TreeError::MissingId)?
                .clone();
            (&mut *value.state.borrow_mut())
                .value
                .deserialize_update_dyn(d.decode_value()?, ctx)?;
            d.decode_end()?;
        }
        Ok(())
    }
}
