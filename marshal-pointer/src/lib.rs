#![feature(allocator_api)]
#![feature(unsize)]
#![feature(coerce_unsized)]
#![feature(raw_ref_op)]
#![feature(dispatch_from_dyn)]
#![feature(core_intrinsics)]
#![feature(arbitrary_self_types)]
#![feature(ptr_metadata)]
#![allow(internal_features)]
#![feature(layout_for_ptr)]
#![feature(hint_assert_unchecked)]

use std::any::{Any, TypeId};
use std::rc::Rc;
use std::sync::Arc;
use std::{rc, sync};

mod arc_inner;
pub mod arc_ref;
pub mod arc_weak_ref;
pub mod boxed;
pub mod empty_arc;
pub mod flat;
mod rc_inner;
pub mod rc_ref;
pub mod rc_weak_ref;
pub mod unique_arc;
pub mod unique_rc;
pub mod empty_rc;

pub trait AsFlatRef {
    type FlatRef: ?Sized;
    fn as_flat_ref(&self) -> &Self::FlatRef;
}

pub trait DerefRaw {
    type RawTarget: ?Sized;
    fn deref_raw(&self) -> *const Self::RawTarget;
}

pub trait RawAny: Any {
    fn raw_type_id(self: *const Self) -> TypeId;
}

impl<T: Any> RawAny for T {
    fn raw_type_id(self: *const Self) -> TypeId {
        TypeId::of::<T>()
    }
}

pub trait DowncastRef<T: ?Sized> {
    fn downcast_ref(&self) -> Option<&T>;
}

pub fn arc_downcast<T: 'static>(arc: Arc<dyn Any>) -> Result<Arc<T>, Arc<dyn Any>> {
    unsafe {
        if (*arc).type_id() == TypeId::of::<T>() {
            Ok(Arc::from_raw(Arc::into_raw(arc) as *const T))
        } else {
            Err(arc)
        }
    }
}

pub fn arc_weak_downcast<T: 'static>(
    weak: sync::Weak<dyn RawAny>,
) -> Result<sync::Weak<T>, sync::Weak<dyn RawAny>> {
    unsafe {
        if weak.as_ptr().raw_type_id() == TypeId::of::<T>() {
            Ok(sync::Weak::from_raw(sync::Weak::into_raw(weak) as *const T))
        } else {
            Err(weak)
        }
    }
}

pub fn rc_downcast<T: 'static>(rc: Rc<dyn Any>) -> Result<Rc<T>, Rc<dyn Any>> {
    unsafe {
        if (*rc).type_id() == TypeId::of::<T>() {
            Ok(Rc::from_raw(Rc::into_raw(rc) as *const T))
        } else {
            Err(rc)
        }
    }
}

pub fn rc_weak_downcast<T: 'static>(
    weak: rc::Weak<dyn RawAny>,
) -> Result<rc::Weak<T>, rc::Weak<dyn RawAny>> {
    unsafe {
        if weak.as_ptr().raw_type_id() == TypeId::of::<T>() {
            Ok(rc::Weak::from_raw(rc::Weak::into_raw(weak) as *const T))
        } else {
            Err(weak)
        }
    }
}

impl dyn RawAny {
    fn downcast_raw_const<T: 'static>(self: *const Self) -> Option<*const T> {
        if self.raw_type_id() == TypeId::of::<T>() {
            Some(self as *const T)
        } else {
            None
        }
    }
    fn downcast_raw_mut<T: 'static>(self: *mut Self) -> Option<*mut T> {
        if self.raw_type_id() == TypeId::of::<T>() {
            Some(self as *mut T)
        } else {
            None
        }
    }
}
