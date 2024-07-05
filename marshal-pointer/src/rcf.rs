use std::any::Any;
use std::borrow::Borrow;
use std::fmt::{Debug, Display, Formatter};
use std::hash::Hash;
use std::marker::Unsize;
use std::ops::{CoerceUnsized, Deref, DispatchFromDyn};
use std::rc;
use std::rc::{Rc, Weak};

use crate::{AsFlatRef, DerefRaw};
use crate::either_rc_ref::RcRef;
use crate::rc_weak_ref::RcWeakRef;

#[derive(Eq, PartialEq, Hash, Default, Ord, PartialOrd)]
pub struct Rcf<T: ?Sized>(Rc<T>);
pub struct RcfWeak<T: ?Sized>(rc::Weak<T>);

impl<T> From<T> for Rcf<T> {
    fn from(value: T) -> Self {
        Rcf(Rc::new(value))
    }
}

impl<T: ?Sized> From<Rc<T>> for Rcf<T> {
    fn from(value: Rc<T>) -> Self {
        Rcf(value)
    }
}

impl<T: ?Sized> From<rc::Weak<T>> for RcfWeak<T> {
    fn from(value: Weak<T>) -> Self {
        RcfWeak(value)
    }
}

impl<T: ?Sized> From<Rcf<T>> for Rc<T> {
    fn from(value: Rcf<T>) -> Self {
        value.0
    }
}

impl<T: ?Sized> From<RcfWeak<T>> for rc::Weak<T> {
    fn from(value: RcfWeak<T>) -> Self {
        value.0
    }
}

impl<T: ?Sized> Deref for Rcf<T> {
    type Target = RcRef<T>;
    fn deref(&self) -> &Self::Target {
        self.0.as_flat_ref()
    }
}

impl<T: ?Sized> AsFlatRef for Rcf<T> {
    type FlatRef = RcRef<T>;
    fn as_flat_ref(&self) -> &Self::FlatRef {
        self.0.as_flat_ref()
    }
}

impl<T: ?Sized> AsFlatRef for RcfWeak<T> {
    type FlatRef = RcWeakRef<T>;
    fn as_flat_ref(&self) -> &Self::FlatRef {
        self.0.as_flat_ref()
    }
}

impl<T: ?Sized> DerefRaw for Rcf<T> {
    type RawTarget = T;
    fn deref_raw(&self) -> *const Self::RawTarget {
        self.0.deref_raw()
    }
}

impl<T: ?Sized> DerefRaw for RcfWeak<T> {
    type RawTarget = T;
    fn deref_raw(&self) -> *const Self::RawTarget {
        self.0.deref_raw()
    }
}

impl<T: ?Sized> Rcf<T> {
    pub fn new(value: T) -> Rcf<T>
    where
        T: Sized,
    {
        Rcf(Rc::new(value))
    }
    pub fn downgrade(this: &Self) -> RcfWeak<T> {
        RcfWeak(Rc::downgrade(&this.0))
    }
    pub fn into_raw(this: Self) -> *const T {
        Rc::into_raw(this.0)
    }
    pub unsafe fn from_raw(ptr: *const T) -> Self {
        Rcf(Rc::from_raw(ptr))
    }
}

impl<T: ?Sized> RcfWeak<T> {
    pub fn into_raw(this: Self) -> *const T {
        rc::Weak::into_raw(this.0)
    }
    pub unsafe fn from_raw(ptr: *const T) -> Self {
        RcfWeak(rc::Weak::from_raw(ptr))
    }
    pub fn upgrade(&self) -> Option<Rcf<T>> {
        Some(Rcf(self.0.upgrade()?))
    }
}

impl Rcf<dyn Any> {
    pub fn downcast<T: Any>(self) -> Result<Rcf<T>, Self> {
        self.0.downcast().map(Rcf).map_err(Rcf)
    }
}

impl<T: ?Sized> AsRef<T> for Rcf<T> {
    fn as_ref(&self) -> &T {
        self.0.as_ref()
    }
}

impl<T: ?Sized> AsRef<RcRef<T>> for Rcf<T> {
    fn as_ref(&self) -> &RcRef<T> {
        self.0.as_flat_ref()
    }
}

impl<T: ?Sized> Borrow<T> for Rcf<T> {
    fn borrow(&self) -> &T {
        self.0.borrow()
    }
}

impl<T: ?Sized> Borrow<RcRef<T>> for Rcf<T> {
    fn borrow(&self) -> &RcRef<T> {
        self.0.as_flat_ref()
    }
}

impl<T: ?Sized> Clone for Rcf<T> {
    fn clone(&self) -> Self {
        Rcf(self.0.clone())
    }
}

impl<T: ?Sized> Clone for RcfWeak<T> {
    fn clone(&self) -> Self {
        RcfWeak(self.0.clone())
    }
}

impl<T, U> CoerceUnsized<Rcf<U>> for Rcf<T>
where
    T: ?Sized + Unsize<U>,
    U: ?Sized,
{
}

impl<T, U> CoerceUnsized<RcfWeak<U>> for RcfWeak<T>
where
    T: ?Sized + Unsize<U>,
    U: ?Sized,
{
}

impl<T, U> DispatchFromDyn<Rcf<U>> for Rcf<T>
where
    T: ?Sized + Unsize<U>,
    U: ?Sized,
{
}

impl<T: ?Sized + Display> Display for Rcf<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}
impl<T: ?Sized + Debug> Debug for Rcf<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

#[cfg(feature = "weak-table")]
impl<T: ?Sized> weak_table::traits::WeakElement for RcfWeak<T> {
    type Strong = Rcf<T>;
    fn new(view: &Self::Strong) -> Self {
        Rcf::downgrade(view)
    }
    fn view(&self) -> Option<Self::Strong> {
        self.upgrade()
    }
}

#[cfg(feature = "weak-table")]
impl<T: ?Sized + Hash + Eq> weak_table::traits::WeakKey for RcfWeak<T> {
    type Key = T;

    fn with_key<F, R>(view: &Self::Strong, f: F) -> R
    where
        F: FnOnce(&Self::Key) -> R,
    {
        f(view)
    }
}
