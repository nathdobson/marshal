use std::any::Any;
use std::collections::HashMap;
use std::sync::Arc;

use marshal::context::Context;
use marshal::de::Deserialize;
use marshal::de::rc::DeserializeArc;
use marshal::decode::{AnyDecoder, DecodeHint, Decoder};
use marshal_shared::de::deserialize_arc;

use crate::de::DeserializeUpdate;
use crate::forest::error::TreeError;
use crate::forest::forest::{Forest, ForestId, ForestRoot, Tree};

// use crate::ser::DeserializeUpdateDyn;

pub(super) struct ForestDeserializerTable {
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

impl<D: Decoder, T: Deserialize<D>> Deserialize<D> for ForestRoot<T> {
    fn deserialize<'p, 'de>(d: AnyDecoder<'p, 'de, D>, mut ctx: Context) -> anyhow::Result<Self> {
        let forest = Forest::new();
        let mut deserializers = ForestDeserializerTable::new(forest.id());
        let mut ctx = ctx.insert_mut_scoped(&mut deserializers);
        let root = T::deserialize(d, ctx.borrow())?;
        Ok(ForestRoot::new_raw(forest, deserializers, root))
    }
}

trait DeserializeUpdateDyn<D: Decoder>: Sync + Send + Any + DeserializeUpdate<D> {}

impl<D: Decoder, T: 'static + Sync + Send + DeserializeUpdate<D>> DeserializeUpdateDyn<D> for T {}

impl<D: Decoder, T: DeserializeUpdate<D>> DeserializeUpdate<D> for ForestRoot<T> {
    fn deserialize_update<'p, 'de>(
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
                0 => root.deserialize_update(d, ctx.reborrow())?,
                1 => {
                    let mut d = d.decode(DecodeHint::Map)?.try_into_map()?;
                    while let Some(mut d) = d.decode_next()? {
                        let key = <usize as Deserialize<D>>::deserialize(
                            d.decode_key()?,
                            ctx.reborrow(),
                        )?;
                        let des: Arc<Tree<dyn DeserializeUpdateDyn<D>>> = (**ctx
                            .reborrow()
                            .get_mut::<ForestDeserializerTable>()?
                            .deserializers
                            .get(&key)
                            .ok_or(TreeError::MissingId)?)
                        .downcast_ref::<Arc<Tree<dyn DeserializeUpdateDyn<D>>>>()
                        .unwrap()
                        .clone();
                        forest
                            .get_mut(&des)
                            .deserialize_update(d.decode_value()?, ctx.reborrow())?;
                        d.decode_end()?;
                    }
                }
                _ => unreachable!(),
            }
        }
        Ok(())
    }
}

impl<D: Decoder, T: 'static + Sync + Send + DeserializeUpdate<D>> DeserializeArc<D> for Tree<T> {
    fn deserialize_arc<'p, 'de>(
        p: AnyDecoder<'p, 'de, D>,
        mut ctx: Context,
    ) -> anyhow::Result<Arc<Self>> {
        let (id, arc) = deserialize_arc::<D, Tree<T>>(p, ctx.reborrow())?;
        ctx.get_mut::<ForestDeserializerTable>()?
            .deserializers
            .insert(
                id,
                Box::new(arc.clone() as Arc<Tree<dyn DeserializeUpdateDyn<D>>>),
            );
        Ok(arc)
    }
}

impl<D: Decoder, T: Deserialize<D>> Deserialize<D> for Tree<T> {
    fn deserialize<'p, 'de>(d: AnyDecoder<'p, 'de, D>, mut ctx: Context) -> anyhow::Result<Self> {
        let tree = T::deserialize(d, ctx.reborrow())?;
        Ok(ctx
            .get_mut::<ForestDeserializerTable>()?
            .forest
            .add_raw(tree))
    }
}
//
// pub trait DynamicDecoder {
//     type DeserializeUpdateDyn: 'static + ?Sized + Sync + Send + Any + Unsize<dyn Sync + Send + Any>;
// }
