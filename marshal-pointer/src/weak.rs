use crate::inner::Inner;
use crate::raw_count::RawCount;
use crate::strong::Strong;
use crate::weak_ref::WeakRef;
use std::alloc::{Allocator, Global, Layout};
use std::ptr::NonNull;
use crate::raw_any::AsFlatRef;

pub struct Weak<C: RawCount, T: ?Sized> {
    inner: NonNull<Inner<C, T>>,
}

impl<C: RawCount, T: ?Sized> Weak<C, T> {
    pub(crate) fn from_inner(inner: *const Inner<C, T>) -> Self {
        Weak {
            inner: NonNull::new(inner as *mut Inner<C, T>).unwrap(),
        }
    }
    pub fn upgrade(&self) -> Option<Strong<C, T>> {
        self.as_flat_ref().strong()
    }
}

impl<C: RawCount, T: ?Sized> Clone for Weak<C, T> {
    fn clone(&self) -> Self {
        self.as_flat_ref().weak()
    }
}

impl<C: RawCount, T: ?Sized> Drop for Weak<C, T> {
    #[inline]
    fn drop(&mut self) {
        unsafe {
            if self.inner.as_ptr().count_raw().decrement_weak() {
                Global.deallocate(
                    self.inner.cast(),
                    Layout::for_value_raw(self.inner.as_ptr()),
                )
            }
        }
    }
}

impl<C: RawCount, T: ?Sized> AsFlatRef for Weak<C, T> {
    type FlatRef = WeakRef<C, T>;
    fn as_flat_ref(&self) -> &Self::FlatRef {
        unsafe { &*(self.inner.as_ptr().into_raw() as *const WeakRef<C, T>) }
    }
}
