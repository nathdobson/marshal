use crate::inner::Inner;
use crate::raw_count::RawCount;
use crate::strong::Strong;
use crate::weak::Weak;
use crate::weak_ref::WeakRef;
use std::marker::PhantomData;
use std::mem;
use std::ops::{Deref, DerefMut};
use std::ptr::NonNull;
use crate::raw_any::AsFlatRef;

pub struct UniqueStrong<C: RawCount, T: ?Sized> {
    inner: NonNull<Inner<C, T>>,
    phantom: PhantomData<Inner<C, T>>,
}

impl<C: RawCount, T: ?Sized> UniqueStrong<C, T> {
    pub fn new(value: T) -> Self
    where
        T: Sized,
    {
        UniqueStrong {
            inner: Inner::new(C::from_counts(0, 1), value),
            phantom: PhantomData,
        }
    }
    pub fn downgrade(&self) -> Weak<C, T> {
        self.as_flat_ref().weak()
    }
    pub fn into_strong(self) -> Strong<C, T> {
        unsafe {
            self.inner
                .as_ptr()
                .count_raw()
                .increment_strong_assume_zero();
            let result = Strong::from_inner(self.inner.as_ptr());
            mem::forget(self);
            result
        }
    }
}

impl<C: RawCount, T: ?Sized> Deref for UniqueStrong<C, T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        unsafe { &*self.inner.as_ptr().into_raw() }
    }
}

impl<C: RawCount, T: ?Sized> DerefMut for UniqueStrong<C, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut *(self.inner.as_ptr().into_raw() as *mut T) }
    }
}

impl<C: RawCount, T: ?Sized> Drop for UniqueStrong<C, T> {
    fn drop(&mut self) {
        mem::drop(Weak::from_inner(self.inner.as_ptr()))
    }
}

impl<C: RawCount, T: ?Sized> AsFlatRef for UniqueStrong<C, T> {
    type FlatRef = WeakRef<C, T>;
    fn as_flat_ref(&self) -> &Self::FlatRef {
        unsafe { &*(self.inner.as_ptr().into_raw() as *const WeakRef<C, T>) }
    }
}
