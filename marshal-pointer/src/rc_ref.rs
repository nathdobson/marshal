use std::any::TypeId;
use std::marker::PhantomData;
use std::ops::Deref;
use std::rc;
use std::rc::Rc;

use crate::{AsFlatRef, DerefRaw, DowncastRef, RawAny};

#[repr(transparent)]
pub struct RcRef<T: ?Sized> {
    phantom: PhantomData<*const ()>,
    inner: T,
}

impl<T: ?Sized> AsFlatRef for Rc<T> {
    type FlatRef = RcRef<T>;

    fn as_flat_ref(&self) -> &Self::FlatRef {
        unsafe { &*((&**self) as *const T as *const RcRef<T>) }
    }
}

impl<T: ?Sized> RcRef<T> {
    pub fn rc(&self) -> Rc<T> {
        unsafe {
            Rc::<Self>::increment_strong_count(self);
            Rc::<T>::from_raw(self as *const RcRef<T> as *const T)
        }
    }
    pub fn weak(&self) -> rc::Weak<T> {
        Rc::downgrade(&self.rc())
    }
}

impl<T: ?Sized> Deref for RcRef<T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<T: ?Sized> DerefRaw for Rc<T> {
    type RawTarget = T;
    fn deref_raw(&self) -> *const Self::RawTarget {
        &**self
    }
}

impl<T: ?Sized> DerefRaw for RcRef<T> {
    type RawTarget = T;
    fn deref_raw(&self) -> *const Self::RawTarget {
        &**self
    }
}

impl<T: 'static> DowncastRef<RcRef<T>> for RcRef<dyn RawAny> {
    fn downcast_ref(&self) -> Option<&RcRef<T>> {
        unsafe {
            if self.deref_raw().raw_type_id() == TypeId::of::<T>() {
                Some(&*(self as *const RcRef<dyn RawAny> as *const RcRef<T>))
            } else {
                None
            }
        }
    }
}

impl<T> AsRef<RcRef<T>> for Rc<T> {
    fn as_ref(&self) -> &RcRef<T> {
        self.as_flat_ref()
    }
}

#[test]
fn test() {
    Rc::new(123).as_flat_ref().rc();
}
