use crate::inner::Inner;
use crate::raw_count::RawCount;
use crate::strong::Strong;
use crate::weak::Weak;
use crate::weak_ref::WeakRef;
use std::mem;
use std::mem::MaybeUninit;
use std::ptr::NonNull;
use crate::raw_any::AsFlatRef;

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
        self.as_flat_ref().weak()
    }
    pub fn into_strong(self, value: T) -> Strong<C, T>
    where
        T: Sized,
    {
        unsafe {
            (self.inner.as_ptr().into_raw() as *mut T).write(value);
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

impl<C: RawCount, T: ?Sized> Drop for EmptyStrong<C, T> {
    fn drop(&mut self) {
        mem::drop(Weak::from_inner(self.inner.as_ptr()))
    }
}

impl<C: RawCount, T: ?Sized> AsFlatRef for EmptyStrong<C, T> {
    type FlatRef = WeakRef<C, T>;
    fn as_flat_ref(&self) -> &Self::FlatRef {
        unsafe { &*(self.inner.as_ptr().into_raw() as *const WeakRef<C, T>) }
    }
}
