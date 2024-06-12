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

mod arc_inner;
pub mod arc_ref;
pub mod arc_weak_ref;
mod flat;
pub mod rc_ref;
pub mod unique_arc;
mod rc_weak_ref;

pub trait AsFlatRef {
    type FlatRef: ?Sized;
    fn as_flat_ref(&self) -> &Self::FlatRef;
}

impl<T: ?Sized> AsFlatRef for Box<T> {
    type FlatRef = T;
    fn as_flat_ref(&self) -> &Self::FlatRef {
        &**self
    }
}
