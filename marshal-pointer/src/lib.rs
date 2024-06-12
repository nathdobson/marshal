#![feature(allocator_api)]
#![feature(unsize)]
#![feature(coerce_unsized)]
#![feature(raw_ref_op)]
#![feature(dispatch_from_dyn)]

use std::ops::Deref;

mod arc_ref;
mod unique_arc;
mod rc_ref;

pub trait PtrRef: Deref {
    type PtrTarget: ?Sized + Deref<Target = Self::Target>;
    fn ptr_ref(&self) -> &Self::PtrTarget;
}
