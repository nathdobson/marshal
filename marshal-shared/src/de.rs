use crate::SharedError;
use marshal::context::Context;
use marshal::de::Deserialize;
use marshal::decode::{AnyDecoder, DecodeHint, Decoder, DecoderView};
use marshal::encode::Encoder;
use marshal::ser::Serialize;
use marshal::Deserialize;
use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::rc::Rc;
use std::sync::Arc;
use weak_table::ptr_weak_key_hash_map::Entry;

#[derive(Default)]
pub struct SharedDeserializeContext {
    rcs: HashMap<usize, Rc<dyn Any>>,
    arcs: HashMap<usize, Arc<dyn Any>>,
}

#[derive(Deserialize)]
struct Shared<X> {
    id: usize,
    inner: Option<X>,
}

pub fn deserialize_rc<'de, D: Decoder<'de>, T: 'static + Deserialize<'de, D>>(
    d: D::AnyDecoder<'_>,
    ctx: &mut Context,
) -> anyhow::Result<Rc<T>> {
    let shared = <Shared<T> as Deserialize<'de, D>>::deserialize(d, ctx)?;
    let shared_ctx = ctx.get_or_default::<SharedDeserializeContext>();
    if let Some(value) = shared.inner {
        let value = Rc::new(value);
        shared_ctx.rcs.insert(shared.id, value.clone());
        Ok(value)
    } else {
        Ok(Rc::downcast(
            shared_ctx
                .rcs
                .get(&shared.id)
                .ok_or(SharedError::UnknownReference(shared.id))?
                .clone(),
        )
        .ok()
        .ok_or(SharedError::TypeMismatch)?)
    }
}

fn arc_downcast<T: 'static>(arc: Arc<dyn Any>) -> Option<Arc<T>> {
    unsafe {
        if (*arc).type_id() == TypeId::of::<T>() {
            Some(Arc::from_raw(Arc::into_raw(arc) as *const T))
        } else {
            None
        }
    }
}

pub fn deserialize_arc<'de, D: Decoder<'de>, T: 'static + Deserialize<'de, D>>(
    d: D::AnyDecoder<'_>,
    ctx: &mut Context,
) -> anyhow::Result<Arc<T>> {
    let shared = <Shared<T> as Deserialize<'de, D>>::deserialize(d, ctx)?;
    let shared_ctx = ctx.get_or_default::<SharedDeserializeContext>();
    if let Some(value) = shared.inner {
        let value = Arc::new(value);
        shared_ctx.arcs.insert(shared.id, value.clone());
        Ok(value)
    } else {
        Ok(arc_downcast(
            shared_ctx
                .arcs
                .get(&shared.id)
                .ok_or(SharedError::UnknownReference(shared.id))?
                .clone(),
        )
        .ok_or(SharedError::TypeMismatch)?)
    }
}

#[macro_export]
macro_rules! derive_deserialize_rc_shared {
    ($ty:ty) => {
        impl<'de, D: $crate::reexports::marshal::decode::Decoder<'de>>
            $crate::reexports::marshal::de::rc::DeserializeRc<'de, D> for $ty
        {
            fn deserialize_rc<'p>(
                p: D::AnyDecoder<'p>,
                ctx: &mut $crate::reexports::marshal::context::Context,
            ) -> anyhow::Result<Rc<Self>> {
                $crate::de::deserialize_rc::<'de, D, Self>(p, ctx)
            }
        }
    };
}

#[macro_export]
macro_rules! derive_deserialize_arc_shared {
    ($ty:ty) => {
        impl<'de, D: $crate::reexports::marshal::decode::Decoder<'de>>
            $crate::reexports::marshal::de::rc::DeserializeArc<'de, D> for $ty
        {
            fn deserialize_arc<'p>(
                p: D::AnyDecoder<'p>,
                ctx: &mut $crate::reexports::marshal::context::Context,
            ) -> $crate::reexports::anyhow::Result<::std::sync::Arc<Self>> {
                $crate::de::deserialize_arc::<'de, D, Self>(p, ctx)
            }
        }
    };
}
