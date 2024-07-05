use crate::inner::Inner;
use crate::raw_count::RawCount;
use crate::strong_ref::StrongRef;
use crate::weak::Weak;
use std::borrow::Borrow;
use std::marker::PhantomData;
use std::mem;
use std::ops::Deref;
use std::ptr::NonNull;
use crate::raw_any::AsFlatRef;

pub struct Strong<C: RawCount, T: ?Sized> {
    inner: NonNull<Inner<C, T>>,
    phantom: PhantomData<Inner<C, T>>,
}

impl<C: RawCount, T: ?Sized> Strong<C, T> {
    pub fn new(value: T) -> Self
    where
        T: Sized,
    {
        Strong {
            inner: Inner::new(C::from_counts(1, 1), value),
            phantom: PhantomData,
        }
    }
    pub(crate) unsafe fn from_inner(inner: *const Inner<C, T>) -> Self {
        Strong {
            inner: NonNull::new(inner as *mut Inner<C, T>).unwrap(),
            phantom: PhantomData,
        }
    }
    pub fn downgrade(this: &Self) -> Weak<C, T> {
        this.as_flat_ref().weak()
    }
    #[inline(never)]
    unsafe fn drop_slow(&mut self) {
        std::ptr::drop_in_place(self.inner.as_ptr().into_raw() as *mut T);
        mem::drop(Weak::<C, T>::from_inner(self.inner.as_ptr()));
    }
}

impl<C: RawCount, T: ?Sized> Clone for Strong<C, T> {
    fn clone(&self) -> Self {
        self.as_flat_ref().strong()
    }
}

impl<C: RawCount, T: ?Sized> Drop for Strong<C, T> {
    #[inline]
    fn drop(&mut self) {
        unsafe {
            if self.inner.as_ptr().count_raw().decrement_strong() {
                self.drop_slow();
            }
        }
    }
}

impl<C: RawCount, T: ?Sized> Deref for Strong<C, T> {
    type Target = StrongRef<C, T>;
    fn deref(&self) -> &Self::Target {
        unsafe { &*(self.inner.as_ptr().into_raw() as *const StrongRef<C, T>) }
    }
}

impl<C: RawCount, T: ?Sized> AsRef<T> for Strong<C, T> {
    fn as_ref(&self) -> &T {
        self.deref().deref()
    }
}

impl<C: RawCount, T: ?Sized> AsRef<StrongRef<C, T>> for Strong<C, T> {
    fn as_ref(&self) -> &StrongRef<C, T> {
        self.deref()
    }
}

impl<C: RawCount, T: ?Sized> Borrow<T> for Strong<C, T> {
    fn borrow(&self) -> &T {
        self.deref()
    }
}

impl<C: RawCount, T: ?Sized> Borrow<StrongRef<C, T>> for Strong<C, T> {
    fn borrow(&self) -> &StrongRef<C, T> {
        self.deref()
    }
}

impl<C: RawCount, T: ?Sized> AsFlatRef for Strong<C, T> {
    type FlatRef = StrongRef<C, T>;
    fn as_flat_ref(&self) -> &Self::FlatRef {
        self.deref()
    }
}
