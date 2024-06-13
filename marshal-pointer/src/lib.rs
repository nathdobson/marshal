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

use crate::boxed::BoxRef;
use std::any::{Any, TypeId};
use std::marker::Unsize;
use std::rc::Rc;
use std::sync::Arc;
use std::{rc, sync};

mod arc_inner;
pub mod arc_ref;
pub mod arc_weak_ref;
pub mod boxed;
pub mod flat;
pub mod rc_ref;
pub mod rc_weak_ref;
pub mod unique_arc;

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
