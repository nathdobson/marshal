use std::{rc, sync};
use std::collections::HashMap;
use std::rc::Rc;
use std::sync::Arc;

use marshal::context::Context;
use marshal::de::Deserialize;
use marshal::decode::{AnyGenDecoder, GenDecoder};
use marshal::Deserialize;
use marshal_pointer::{arc_downcast, arc_weak_downcast, RawAny, rc_downcast, rc_weak_downcast};
use marshal_pointer::empty_arc::EmptyArc;
use marshal_pointer::empty_rc::EmptyRc;

use crate::SharedError;

struct ArcState {
    weak: sync::Weak<dyn RawAny + Sync + Send>,
    uninit: Option<EmptyArc<dyn Sync + Send + RawAny>>,
    init: Option<Arc<dyn Sync + Send + RawAny>>,
}

struct RcState {
    weak: rc::Weak<dyn RawAny>,
    uninit: Option<EmptyRc<dyn RawAny>>,
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
        let uninit = EmptyArc::<T>::new();
        let weak = EmptyArc::downgrade(&uninit);
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
        let uninit = EmptyArc::downcast::<T>(uninit)
            .ok()
            .ok_or(SharedError::TypeMismatch)?;
        let init = EmptyArc::into_arc(uninit, value);
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
        let uninit = EmptyRc::<T>::new();
        let weak = EmptyRc::downgrade(&uninit);
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
        let uninit = EmptyRc::downcast::<T>(uninit)
            .ok()
            .ok_or(SharedError::TypeMismatch)?;
        let init = EmptyRc::into_rc(uninit, value);
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

pub fn deserialize_arc<'p, 'de, D: GenDecoder, T: 'static + Sync + Send + Deserialize<D>>(
    d: AnyGenDecoder<'p, 'de, D>,
    mut ctx: Context,
) -> anyhow::Result<(usize, Arc<T>)> {
    let shared = <Shared<T> as Deserialize<D>>::deserialize(d, ctx.reborrow())?;
    let shared_ctx = ctx.get_mut::<SharedArcDeserializeContext>()?;
    if let Some(value) = shared.inner {
        let state = shared_ctx
            .shared
            .entry(shared.id)
            .or_insert_with(|| ArcState::new_uninit::<T>());
        Ok((shared.id, state.init(value)?))
    } else {
        Ok((
            shared.id,
            shared_ctx
                .shared
                .get(&shared.id)
                .ok_or(SharedError::UnknownReference)?
                .arc::<T>()?,
        ))
    }
}
pub fn deserialize_rc<'de, D: GenDecoder, T: 'static + Deserialize<D>>(
    d: AnyGenDecoder<'_, 'de, D>,
    mut ctx: Context,
) -> anyhow::Result<Rc<T>> {
    let shared = <Shared<T> as Deserialize<D>>::deserialize(d, ctx.reborrow())?;
    let shared_ctx = ctx.get_mut::<SharedRcDeserializeContext>()?;
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
    'p,
    'de,
    D: GenDecoder,
    T: 'static + Sync + Send + Deserialize<D>,
>(
    d: AnyGenDecoder<'p, 'de, D>,
    mut ctx: Context,
) -> anyhow::Result<(usize, sync::Weak<T>)> {
    let id = <usize as Deserialize<D>>::deserialize(d, ctx.reborrow())?;
    let shared_ctx = ctx.get_mut::<SharedArcDeserializeContext>()?;
    Ok((
        id,
        shared_ctx
            .shared
            .entry(id)
            .or_insert_with(|| ArcState::new_uninit::<T>())
            .weak()?,
    ))
}

pub fn deserialize_rc_weak<'de, D: GenDecoder, T: 'static + Deserialize< D>>(
    d: AnyGenDecoder<'_, 'de, D>,
    mut ctx: Context,
) -> anyhow::Result<rc::Weak<T>> {
    let id = <usize as Deserialize<D>>::deserialize(d, ctx.reborrow())?;
    let shared_ctx = ctx.get_mut::<SharedRcDeserializeContext>()?;
    shared_ctx
        .shared
        .entry(id)
        .or_insert_with(|| RcState::new_uninit::<T>())
        .weak()
}

#[macro_export]
macro_rules! derive_deserialize_rc_shared {
    ($ty:ty) => {
        impl<D: $crate::reexports::marshal::decode::GenDecoder>
            $crate::reexports::marshal::de::rc::DeserializeRc<D> for $ty
        {
            fn deserialize_rc<'p, 'de>(
                d: $crate::reexports::marshal::decode::AnyGenDecoder<'p, 'de, D>,
                mut ctx: $crate::reexports::marshal::context::Context,
            ) -> $crate::reexports::anyhow::Result<::std::rc::Rc<Self>> {
                $crate::de::deserialize_rc::<D, Self>(d, ctx)
            }
        }
    };
}

#[macro_export]
macro_rules! derive_deserialize_rc_weak_shared {
    ($ty:ty) => {
        impl<D: $crate::reexports::marshal::decode::GenDecoder>
            $crate::reexports::marshal::de::rc::DeserializeRcWeak<D> for $ty
        {
            fn deserialize_rc_weak<'p, 'de>(
                d: $crate::reexports::marshal::decode::AnyGenDecoder<'p, 'de, D>,
                mut ctx: $crate::reexports::marshal::context::Context,
            ) -> $crate::reexports::anyhow::Result<::std::rc::Weak<Self>> {
                $crate::de::deserialize_rc_weak::<D, Self>(d, ctx)
            }
        }
    };
}

#[macro_export]
macro_rules! derive_deserialize_arc_weak_shared {
    ($ty:ty) => {
        impl<D: $crate::reexports::marshal::decode::GenDecoder>
            $crate::reexports::marshal::de::rc::DeserializeArcWeak<D> for $ty
        {
            fn deserialize_arc_weak<'p, 'de>(
                p: $crate::reexports::marshal::decode::AnyGenDecoder<'p, 'de, D>,
                ctx: $crate::reexports::marshal::context::Context,
            ) -> $crate::reexports::anyhow::Result<::std::sync::Weak<Self>> {
                ::std::result::Result::Ok($crate::de::deserialize_arc_weak::<D, Self>(p, ctx)?.1)
            }
        }
    };
}

#[macro_export]
macro_rules! derive_deserialize_arc_shared {
    ($ty:ty) => {
        impl<D: $crate::reexports::marshal::decode::GenDecoder>
            $crate::reexports::marshal::de::rc::DeserializeArc<D> for $ty
        {
            fn deserialize_arc<'p, 'de>(
                p: $crate::reexports::marshal::decode::AnyGenDecoder<'p, 'de, D>,
                ctx: $crate::reexports::marshal::context::Context,
            ) -> $crate::reexports::anyhow::Result<::std::sync::Arc<Self>> {
                ::std::result::Result::Ok($crate::de::deserialize_arc::<D, Self>(p, ctx)?.1)
            }
        }
    };
}
