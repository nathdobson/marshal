use crate::AsFlatRef;
use std::marker::PhantomData;
use std::ops::Deref;
use std::sync::Arc;

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
}

impl<T: ?Sized> Deref for ArcRef<T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

#[test]
fn test() {
    Arc::new(123).as_flat_ref().arc();
}
