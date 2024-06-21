use std::any::Any;
use std::sync;
use std::sync::Arc;

use marshal::context::Context;
use marshal::encode::AnyEncoder;
use marshal::reexports::marshal_pointer::arc_ref::ArcRef;
use marshal::ser::rc::SerializeArc;
use marshal::ser::Serialize;
use marshal_bin::encode::full::BinEncoder;
use marshal_shared::ser::SharedSerializeContext;

use crate::ser::SerializeUpdateDyn;
use crate::tree::ser::SerializeForest;
use crate::tree::Tree;

pub trait SerializeUpdateBin: for<'s> SerializeUpdateDyn<BinEncoder<'s>> {}

impl<T: ?Sized + for<'s> SerializeUpdateDyn<BinEncoder<'s>>> SerializeUpdateBin for T {}

impl<'s, T: 'static + Serialize<BinEncoder<'s>>> SerializeArc<BinEncoder<'s>> for Tree<T> {
    fn serialize_arc<'p>(
        this: &ArcRef<Self>,
        e: AnyEncoder<'p, BinEncoder<'s>>,
        ctx: &mut Context,
    ) -> anyhow::Result<()> {
        let forest = ctx.get::<SerializeForest<dyn SerializeUpdateBin>>()?;
        this.forest.get_or_init(|| forest.queue.clone());
        SharedSerializeContext::<sync::Weak<Tree<dyn Any>>>::serialize_strong(
            this,
            this.weak(),
            e,
            ctx,
        )?;
        Ok(())
    }
}
