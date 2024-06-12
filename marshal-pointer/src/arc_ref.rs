use std::{
    fmt::Debug,
    hash::Hash,
    marker::PhantomData,
    mem,
    ops::Deref,
    sync::Arc,
};

use crate::PtrRef;

#[repr(transparent)]
pub struct ArcRef<T: ?Sized> {
    phantom: PhantomData<*const ()>,
    inner: T,
}

impl<T: ?Sized> PtrRef for Arc<T> {
    type PtrTarget = ArcRef<T>;

    fn ptr_ref(&self) -> &Self::PtrTarget {
        unsafe { mem::transmute::<&T, &ArcRef<T>>(&**self) }
    }
}

unsafe impl<T: ?Sized> Sync for ArcRef<T> where T: Sync + Send {}
unsafe impl<T: ?Sized> Send for ArcRef<T> where T: Sync + Send {}

impl<T: ?Sized> ArcRef<T> {
    pub fn arc(&self) -> Arc<T> {
        unsafe {
            Arc::<Self>::increment_strong_count(self);
            Arc::<T>::from_raw(self as *const ArcRef<T> as *const T)
        }
    }
}

impl<T: ?Sized> Deref for ArcRef<T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}
