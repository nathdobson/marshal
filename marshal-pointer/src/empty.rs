use crate::inner::Inner;
use crate::raw_count::RawCount;
use crate::weak::Weak;
use std::mem::MaybeUninit;
use std::ptr::NonNull;

pub struct EmptyStrong<C: RawCount, T: ?Sized> {
    inner: NonNull<Inner<C, T>>,
}

impl<C: RawCount, T: ?Sized> EmptyStrong<C, T> {
    pub fn new() -> Self
    where
        T: Sized,
    {
        EmptyStrong {
            inner: Inner::new(C::from_counts(0, 1), MaybeUninit::<T>::uninit()).cast(),
        }
    }
    pub fn downgrade(&self) -> Weak<C, T> {
        unsafe {
            self.inner.as_ptr().count_raw().increment_weak();
            Weak::from_inner(self.inner.as_ptr())
        }
    }
}
