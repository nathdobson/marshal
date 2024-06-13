use std::any::TypeId;
use crate::{AsFlatRef, DerefRaw, DowncastRef, RawAny};
use std::ops::Deref;

#[repr(transparent)]
pub struct BoxRef<T: ?Sized>(T);

impl<T: ?Sized> AsFlatRef for Box<T> {
    type FlatRef = BoxRef<T>;
    fn as_flat_ref(&self) -> &Self::FlatRef {
        unsafe { &*((&**self) as *const T as *const BoxRef<T>) }
    }
}

impl<T: ?Sized> Deref for BoxRef<T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T: ?Sized> DerefRaw for BoxRef<T> {
    type RawTarget = T;
    fn deref_raw(&self) -> *const Self::RawTarget {
        &**self
    }
}

impl<T: ?Sized> DerefRaw for Box<T> {
    type RawTarget = T;
    fn deref_raw(&self) -> *const Self::RawTarget {
        &**self
    }
}

impl<T: 'static> DowncastRef<BoxRef<T>> for BoxRef<dyn RawAny> {
    fn downcast_ref(&self) -> Option<&BoxRef<T>> {
        unsafe {
            if self.deref_raw().raw_type_id() == TypeId::of::<T>() {
                Some(&*(self as *const BoxRef<dyn RawAny> as *const BoxRef<T>))
            } else {
                None
            }
        }
    }
}
