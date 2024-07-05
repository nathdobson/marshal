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
#![feature(alloc_layout_extra)]
#![feature(set_ptr_value)]
#![feature(never_type)]
#![feature(strict_provenance)]

use crate::empty::EmptyStrong;
use crate::raw_arc::RawArc;
use crate::raw_rc::RawRc;
use crate::strong::Strong;
use crate::strong_ref::StrongRef;
use crate::unique::UniqueStrong;
use crate::weak::Weak;
use crate::weak_ref::WeakRef;

pub mod boxed;
pub mod empty;
pub mod inner;
pub mod raw_any;
pub mod raw_arc;
pub mod raw_count;
pub mod raw_rc;
pub mod strong;
pub mod strong_ref;
pub mod unique;
pub mod weak;
pub mod weak_ref;

pub type Arcf<T> = Strong<RawArc, T>;
pub type ArcfWeak<T> = Weak<RawArc, T>;
pub type ArcfRef<T> = StrongRef<RawArc, T>;
pub type ArcfWeakRef<T> = WeakRef<RawArc, T>;
pub type UniqueArcf<T> = UniqueStrong<RawArc, T>;
pub type EmptyArcf<T> = EmptyStrong<RawArc, T>;

pub type Rcf<T> = Strong<RawRc, T>;
pub type RcfWeak<T> = Weak<RawRc, T>;
pub type RcfRef<T> = StrongRef<RawRc, T>;
pub type RcfWeakRef<T> = WeakRef<RawRc, T>;
pub type UniqueRcf<T> = UniqueStrong<RawRc, T>;
pub type EmptyRcf<T> = EmptyStrong<RawRc, T>;

pub trait AsFlatRef {
    type FlatRef: ?Sized;
    fn as_flat_ref(&self) -> &Self::FlatRef;
}
