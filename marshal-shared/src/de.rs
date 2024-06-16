use crate::SharedError;
use marshal::context::Context;
use marshal::de::Deserialize;
use marshal::decode::Decoder;
use marshal::Deserialize;
use marshal_pointer::unique_arc::UniqueArc;
use marshal_pointer::unique_rc::UniqueRc;
use marshal_pointer::{arc_downcast, arc_weak_downcast, rc_downcast, rc_weak_downcast, RawAny};
use std::any::{Any, TypeId};
use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::mem::MaybeUninit;
use std::rc::Rc;
use std::sync::Arc;
use std::{rc, sync};

struct ArcState {
    weak: sync::Weak<dyn RawAny + Sync + Send>,
    uninit: Option<UniqueArc<dyn Sync + Send + RawAny>>,
    init: Option<Arc<dyn Sync + Send + RawAny>>,
}

struct RcState {
    weak: rc::Weak<dyn RawAny>,
    uninit: Option<UniqueRc<dyn RawAny>>,
    init: Option<Rc<dyn RawAny>>,
}

#[derive(Default)]
pub struct SharedArcDeserializeContext {
    shared: HashMap<usize, ArcState>,
}

#[derive(Default)]
pub struct SharedRcDeserializeContext {
    shared: HashMap<usize, RcState>,
}

impl ArcState {
    pub fn new_uninit<T: 'static + Sync + Send>() -> Self {
        let uninit = UniqueArc::<MaybeUninit<T>>::new_uninit();
        let weak = UniqueArc::downgrade_uninit(&uninit);
        ArcState {
            weak,
            uninit: Some(uninit),
            init: None,
        }
    }
    pub fn new<T: 'static + Sync + Send>(init: Arc<T>) -> Self {
        ArcState {
            weak: Arc::<T>::downgrade(&init),
            uninit: None,
            init: Some(init),
        }
    }
    pub fn init<T: 'static + Sync + Send>(&mut self, value: T) -> anyhow::Result<Arc<T>> {
        let uninit = self.uninit.take().ok_or(SharedError::DoubleDefinition)?;
        let uninit = UniqueArc::downcast::<MaybeUninit<T>>(uninit)
            .ok()
            .ok_or(SharedError::TypeMismatch)?;
        let init = uninit.init(value);
        self.init = Some(init.clone());
        Ok(init)
    }
    pub fn weak<T: 'static + Sync + Send>(&self) -> anyhow::Result<sync::Weak<T>> {
        Ok(arc_weak_downcast::<T>(self.weak.clone())
            .ok()
            .ok_or(SharedError::TypeMismatch)?)
    }
    pub fn arc<T: 'static + Sync + Send>(&self) -> anyhow::Result<Arc<T>> {
        Ok(
            arc_downcast::<T>(self.init.clone().ok_or(SharedError::MissingDefinition)?)
                .ok()
                .ok_or(SharedError::TypeMismatch)?,
        )
    }
}

impl RcState {
    pub fn new_uninit<T: 'static>() -> Self {
        let uninit = UniqueRc::<MaybeUninit<T>>::new_uninit();
        let weak = UniqueRc::downgrade_uninit(&uninit);
        RcState {
            weak,
            uninit: Some(uninit),
            init: None,
        }
    }
    pub fn new<T: 'static>(init: Rc<T>) -> Self {
        RcState {
            weak: Rc::<T>::downgrade(&init),
            uninit: None,
            init: Some(init),
        }
    }
    pub fn init<T: 'static>(&mut self, value: T) -> anyhow::Result<Rc<T>> {
        let uninit = self.uninit.take().ok_or(SharedError::DoubleDefinition)?;
        let uninit = UniqueRc::downcast::<MaybeUninit<T>>(uninit)
            .ok()
            .ok_or(SharedError::TypeMismatch)?;
        let init = uninit.init(value);
        self.init = Some(init.clone());
        Ok(init)
    }
    pub fn weak<T: 'static>(&self) -> anyhow::Result<rc::Weak<T>> {
        Ok(rc_weak_downcast::<T>(self.weak.clone())
            .ok()
            .ok_or(SharedError::TypeMismatch)?)
    }
    pub fn rc<T: 'static>(&self) -> anyhow::Result<Rc<T>> {
        Ok(
            rc_downcast::<T>(self.init.clone().ok_or(SharedError::MissingDefinition)?)
                .ok()
                .ok_or(SharedError::TypeMismatch)?,
        )
    }
}

#[derive(Deserialize)]
struct Shared<X> {
    id: usize,
    inner: Option<X>,
}

pub fn deserialize_arc<'de, D: Decoder<'de>, T: 'static + Sync + Send + Deserialize<'de, D>>(
    d: D::AnyDecoder<'_>,
    ctx: &mut Context,
) -> anyhow::Result<Arc<T>> {
    let shared = <Shared<T> as Deserialize<'de, D>>::deserialize(d, ctx)?;
    let shared_ctx = ctx.get_or_default::<SharedArcDeserializeContext>();
    if let Some(value) = shared.inner {
        let state = shared_ctx
            .shared
            .entry(shared.id)
            .or_insert_with(|| ArcState::new_uninit::<T>());
        state.init(value)
    } else {
        Ok(shared_ctx
            .shared
            .get(&shared.id)
            .ok_or(SharedError::UnknownReference)?
            .arc::<T>()?)
    }
}
pub fn deserialize_rc<'de, D: Decoder<'de>, T: 'static + Deserialize<'de, D>>(
    d: D::AnyDecoder<'_>,
    ctx: &mut Context,
) -> anyhow::Result<Rc<T>> {
    let shared = <Shared<T> as Deserialize<'de, D>>::deserialize(d, ctx)?;
    let shared_ctx = ctx.get_or_default::<SharedRcDeserializeContext>();
    if let Some(value) = shared.inner {
        let state = shared_ctx
            .shared
            .entry(shared.id)
            .or_insert_with(|| RcState::new_uninit::<T>());
        state.init(value)
    } else {
        Ok(shared_ctx
            .shared
            .get(&shared.id)
            .ok_or(SharedError::UnknownReference)?
            .rc::<T>()?)
    }
}

pub fn deserialize_arc_weak<
    'de,
    D: Decoder<'de>,
    T: 'static + Sync + Send + Deserialize<'de, D>,
>(
    d: D::AnyDecoder<'_>,
    ctx: &mut Context,
) -> anyhow::Result<sync::Weak<T>> {
    let id = <usize as Deserialize<D>>::deserialize(d, ctx)?;
    let shared_ctx = ctx.get_or_default::<SharedArcDeserializeContext>();
    shared_ctx
        .shared
        .entry(id)
        .or_insert_with(|| ArcState::new_uninit::<T>())
        .weak()
}

pub fn deserialize_rc_weak<'de, D: Decoder<'de>, T: 'static + Deserialize<'de, D>>(
    d: D::AnyDecoder<'_>,
    ctx: &mut Context,
) -> anyhow::Result<rc::Weak<T>> {
    todo!();
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
                $crate::de::deserialize_rc::<D, Self>(p, ctx)
            }
        }
    };
}

#[macro_export]
macro_rules! derive_deserialize_rc_weak_shared {
    ($ty:ty) => {
        impl<'de, D: $crate::reexports::marshal::decode::Decoder<'de>>
            $crate::reexports::marshal::de::rc::DeserializeRcWeak<'de, D> for $ty
        {
            fn deserialize_rc_weak<'p>(
                p: D::AnyDecoder<'p>,
                ctx: &mut $crate::reexports::marshal::context::Context,
            ) -> anyhow::Result<::std::rc::Weak<Self>> {
                $crate::de::deserialize_rc_weak::<D, Self>(p, ctx)
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
                $crate::de::deserialize_arc::<D, Self>(p, ctx)
            }
        }
    };
}