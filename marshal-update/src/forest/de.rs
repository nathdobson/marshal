use std::any::Any;
use std::collections::HashMap;
use std::marker::Unsize;
use std::sync::Arc;

use marshal::context::Context;
use marshal::de::rc::DeserializeArc;
use marshal::de::Deserialize;
use marshal::decode::{AnyDecoder, DecodeHint, Decoder};
use marshal_shared::de::deserialize_arc;

use crate::de::DeserializeUpdate;
use crate::forest::error::TreeError;
use crate::forest::forest::{Forest, ForestId, ForestRoot, Tree};
use crate::ser::DeserializeUpdateDyn;

pub struct ForestDeserializerTable {
    forest: ForestId,
    deserializers: HashMap<usize, Box<dyn Sync + Send + Any>>,
}

impl ForestDeserializerTable {
    pub fn new(forest: ForestId) -> ForestDeserializerTable {
        ForestDeserializerTable {
            forest,
            deserializers: HashMap::new(),
        }
    }
}

impl<'de, D: Decoder<'de>, T: Deserialize<'de, D>> Deserialize<'de, D> for ForestRoot<T> {
    fn deserialize<'p>(d: AnyDecoder<'p, 'de, D>, mut ctx: Context) -> anyhow::Result<Self> {
        let forest = Forest::new();
        let mut deserializers = ForestDeserializerTable::new(forest.id());
        let mut ctx = ctx.insert_mut_scoped(&mut deserializers);
        let root = T::deserialize(d, ctx.borrow())?;
        Ok(ForestRoot::new_raw(forest, deserializers, root))
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
        let (root, forest, deserializers) = self.view_mut_internal();
        let mut ctx = ctx.clone_scoped();
        ctx.insert_mut::<ForestDeserializerTable>(deserializers);
        let mut ctx = ctx.borrow();
        let mut d = d.decode_struct_helper("ForestRoot", &["root", "trees"])?;
        while let Some((field, mut d)) = d.next()? {
            match field {
                0 => root.deserialize_update(d.decode_field()?, ctx.reborrow())?,
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
                        forest
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
        Ok(ctx.get_mut::<ForestDeserializerTable>()?.forest.add(tree))
    }
}

pub trait DynamicDecoder {
    type DeserializeUpdateDyn: 'static + ?Sized + Sync + Send + Any + Unsize<dyn Sync + Send + Any>;
}
