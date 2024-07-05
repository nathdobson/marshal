use crate::inner::Inner;
use crate::raw_count::RawCount;
use crate::strong::Strong;
use std::alloc::{Allocator, Global, Layout};
use std::ptr::NonNull;

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
        unsafe {
            if self.inner.as_ptr().count_raw().increment_strong_if_non_zero() {
                Some(Strong::from_inner(self.inner.as_ptr()))
            } else {
                None
            }
        }
    }
}

impl<C: RawCount, T: ?Sized> Clone for Weak<C, T> {
    fn clone(&self) -> Self {
        unsafe {
            self.inner.as_ptr().count_raw().increment_weak();
            Weak::from_inner(self.inner.as_ptr())
        }
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
