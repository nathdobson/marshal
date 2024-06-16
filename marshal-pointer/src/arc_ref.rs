use std::any::TypeId;
use std::marker::PhantomData;
use std::ops::Deref;
use std::sync;
use std::sync::Arc;

use crate::{AsFlatRef, DerefRaw, DowncastRef, RawAny};

#[repr(transparent)]
pub struct ArcRef<T: ?Sized> {
    phantom: PhantomData<*const ()>,
    inner: T,
}

impl<T: ?Sized> AsFlatRef for Arc<T> {
    type FlatRef = ArcRef<T>;
    fn as_flat_ref(&self) -> &Self::FlatRef {
        unsafe { &*((&**self) as *const T as *const ArcRef<T>) }
    }
}

unsafe impl<T: ?Sized> Sync for ArcRef<T> where T: Sync + Send {}
unsafe impl<T: ?Sized> Send for ArcRef<T> where T: Sync + Send {}

impl<T: ?Sized> ArcRef<T> {
    pub fn arc(&self) -> Arc<T> {
        unsafe {
            let ptr: *const T = &**self;
            Arc::increment_strong_count(ptr);
            Arc::from_raw(ptr)
        }
    }
    pub fn weak(&self) -> sync::Weak<T> {
        Arc::downgrade(&self.arc())
    }
}

impl<T: ?Sized> Deref for ArcRef<T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<T: ?Sized> DerefRaw for Arc<T> {
    type RawTarget = T;
    fn deref_raw(&self) -> *const Self::RawTarget {
        &**self
    }
}

impl<T: ?Sized> DerefRaw for ArcRef<T> {
    type RawTarget = T;
    fn deref_raw(&self) -> *const Self::RawTarget {
        &**self
    }
}

impl<T: 'static> DowncastRef<ArcRef<T>> for ArcRef<dyn RawAny> {
    fn downcast_ref(&self) -> Option<&ArcRef<T>> {
        unsafe {
            if self.deref_raw().raw_type_id() == TypeId::of::<T>() {
                Some(&*(self as *const ArcRef<dyn RawAny> as *const ArcRef<T>))
            } else {
                None
            }
        }
    }
}

#[test]
fn test() {
    Arc::new(123).as_flat_ref().arc();
}
