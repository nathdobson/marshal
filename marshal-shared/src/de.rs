use std::collections::HashMap;

use marshal::context::Context;
use marshal::de::Deserialize;
use marshal::decode::{AnyDecoder, Decoder};
use marshal::Deserialize;
use marshal_pointer::{Arcf, ArcfWeak, EmptyArcf, EmptyRcf, Rcf, RcfWeak};
use marshal_pointer::raw_any::RawAny;

use crate::SharedError;

struct ArcState {
    weak: ArcfWeak<dyn RawAny + Sync + Send>,
    uninit: Option<EmptyArcf<dyn Sync + Send + RawAny>>,
    init: Option<Arcf<dyn Sync + Send + RawAny>>,
}

struct RcState {
    weak: RcfWeak<dyn RawAny>,
    uninit: Option<EmptyRcf<dyn RawAny>>,
    init: Option<Rcf<dyn RawAny>>,
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
        let uninit = EmptyArcf::<T>::new();
        let weak = EmptyArcf::downgrade(&uninit);
        ArcState {
            weak,
            uninit: Some(uninit),
            init: None,
        }
    }
    pub fn init<T: 'static + Sync + Send>(&mut self, value: T) -> anyhow::Result<Arcf<T>> {
        let uninit = self.uninit.take().ok_or(SharedError::DoubleDefinition)?;
        let uninit = EmptyArcf::downcast::<T>(uninit)
            .ok()
            .ok_or(SharedError::TypeMismatch)?;
        let init = EmptyArcf::into_strong(uninit, value);
        self.init = Some(init.clone());
        Ok(init)
    }
    pub fn weak<T: 'static + Sync + Send>(&self) -> anyhow::Result<ArcfWeak<T>> {
        Ok(ArcfWeak::downcast(self.weak.clone())
            .ok()
            .ok_or(SharedError::TypeMismatch)?)
    }
    pub fn arc<T: 'static + Sync + Send>(&self) -> anyhow::Result<Arcf<T>> {
        Ok(
            ((self.init.clone().ok_or(SharedError::MissingDefinition)?) as Arcf<dyn RawAny>)
                .downcast::<T>()
                .map_err(|e| e.map(|_| ()))?,
        )
    }
}

impl RcState {
    pub fn new_uninit<T: 'static>() -> Self {
        let uninit = EmptyRcf::<T>::new();
        let weak = EmptyRcf::downgrade(&uninit);
        RcState {
            weak,
            uninit: Some(uninit),
            init: None,
        }
    }
    pub fn init<T: 'static>(&mut self, value: T) -> anyhow::Result<Rcf<T>> {
        let uninit = self.uninit.take().ok_or(SharedError::DoubleDefinition)?;
        let uninit = EmptyRcf::downcast::<T>(uninit)
            .ok()
            .ok_or(SharedError::TypeMismatch)?;
        let init = EmptyRcf::into_strong(uninit, value);
        self.init = Some(init.clone());
        Ok(init)
    }
    pub fn weak<T: 'static>(&self) -> anyhow::Result<RcfWeak<T>> {
        Ok(self
            .weak
            .clone()
            .downcast::<T>()
            .map_err(|e| e.map(|_| ()))?)
    }
    pub fn rc<T: 'static>(&self) -> anyhow::Result<Rcf<T>> {
        Ok(self
            .init
            .clone()
            .ok_or(SharedError::MissingDefinition)?
            .downcast::<T>()
            .map_err(|e| e.map(|_| ()))?)
    }
}

#[derive(Deserialize)]
struct Shared<X> {
    id: usize,
    inner: Option<X>,
}

pub fn deserialize_arc<'p, 'de, D: Decoder, T: 'static + Sync + Send + Deserialize<D>>(
    d: AnyDecoder<'p, 'de, D>,
    mut ctx: Context,
) -> anyhow::Result<(usize, Arcf<T>)> {
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
pub fn deserialize_rc<'de, D: Decoder, T: 'static + Deserialize<D>>(
    d: AnyDecoder<'_, 'de, D>,
    mut ctx: Context,
) -> anyhow::Result<Rcf<T>> {
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

pub fn deserialize_arc_weak<'p, 'de, D: Decoder, T: 'static + Sync + Send + Deserialize<D>>(
    d: AnyDecoder<'p, 'de, D>,
    mut ctx: Context,
) -> anyhow::Result<(usize, ArcfWeak<T>)> {
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

pub fn deserialize_rc_weak<'de, D: Decoder, T: 'static + Deserialize<D>>(
    d: AnyDecoder<'_, 'de, D>,
    mut ctx: Context,
) -> anyhow::Result<RcfWeak<T>> {
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
        impl<D: $crate::reexports::marshal::decode::Decoder>
            $crate::reexports::marshal::de::rc::DeserializeRc<D> for $ty
        {
            fn deserialize_rc<'p, 'de>(
                d: $crate::reexports::marshal::decode::AnyDecoder<'p, 'de, D>,
                mut ctx: $crate::reexports::marshal::context::Context,
            ) -> $crate::reexports::anyhow::Result<$crate::reexports::marshal_pointer::Rcf<Self>>
            {
                $crate::de::deserialize_rc::<D, Self>(d, ctx)
            }
        }
    };
}

#[macro_export]
macro_rules! derive_deserialize_rc_weak_shared {
    ($ty:ty) => {
        impl<D: $crate::reexports::marshal::decode::Decoder>
            $crate::reexports::marshal::de::rc::DeserializeRcWeak<D> for $ty
        {
            fn deserialize_rc_weak<'p, 'de>(
                d: $crate::reexports::marshal::decode::AnyDecoder<'p, 'de, D>,
                mut ctx: $crate::reexports::marshal::context::Context,
            ) -> $crate::reexports::anyhow::Result<$crate::reexports::marshal_pointer::RcfWeak<Self>> {
                $crate::de::deserialize_rc_weak::<D, Self>(d, ctx)
            }
        }
    };
}

#[macro_export]
macro_rules! derive_deserialize_arc_weak_shared {
    ($ty:ty) => {
        impl<D: $crate::reexports::marshal::decode::Decoder>
            $crate::reexports::marshal::de::rc::DeserializeArcWeak<D> for $ty
        {
            fn deserialize_arc_weak<'p, 'de>(
                p: $crate::reexports::marshal::decode::AnyDecoder<'p, 'de, D>,
                ctx: $crate::reexports::marshal::context::Context,
            ) -> $crate::reexports::anyhow::Result<$crate::reexports::marshal_pointer::ArcfWeak<Self>> {
                ::std::result::Result::Ok($crate::de::deserialize_arc_weak::<D, Self>(p, ctx)?.1)
            }
        }
    };
}

#[macro_export]
macro_rules! derive_deserialize_arc_shared {
    ($ty:ty) => {
        impl<D: $crate::reexports::marshal::decode::Decoder>
            $crate::reexports::marshal::de::rc::DeserializeArc<D> for $ty
        {
            fn deserialize_arc<'p, 'de>(
                p: $crate::reexports::marshal::decode::AnyDecoder<'p, 'de, D>,
                ctx: $crate::reexports::marshal::context::Context,
            ) -> $crate::reexports::anyhow::Result<$crate::reexports::marshal_pointer::Arcf<Self>>
            {
                ::std::result::Result::Ok($crate::de::deserialize_arc::<D, Self>(p, ctx)?.1)
            }
        }
    };
}
