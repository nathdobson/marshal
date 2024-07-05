use crate::inner::Inner;
use crate::raw_any::{DerefRaw, DowncastError, RawAny};
use crate::raw_count::RawCount;
use crate::strong_ref::StrongRef;
use crate::weak::Weak;
use crate::AsFlatRef;
use std::borrow::Borrow;
use std::fmt::{Debug, Formatter};
use std::marker::{PhantomData, Unsize};
use std::mem;
use std::ops::{CoerceUnsized, Deref};
use std::ptr::NonNull;

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
    pub(crate) unsafe fn into_inner(self) -> *const Inner<C, T> {
        let inner = self.inner.as_ptr();
        mem::forget(self);
        inner
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

impl<C: RawCount, T: ?Sized + Unsize<U>, U: ?Sized> CoerceUnsized<Strong<C, U>> for Strong<C, T> {}

impl<C: RawCount> Strong<C, dyn RawAny> {
    pub fn downcast<T: 'static>(self) -> Result<Strong<C, T>, DowncastError<Self>> {
        unsafe {
            match self.into_inner().downcast_inner::<T>() {
                Ok(x) => Ok(Strong::from_inner(x)),
                Err(e) => Err(e.map(|e| Strong::from_inner(e))),
            }
        }
    }
}

unsafe impl<C: RawCount, T: ?Sized> Sync for Strong<C, T>
where
    T: Sync + Send,
    C: Sync + Send,
{
}

unsafe impl<C: RawCount, T: ?Sized> Send for Strong<C, T>
where
    T: Sync + Send,
    C: Sync + Send,
{
}

impl<C: RawCount, T: ?Sized> DerefRaw for Strong<C, T> {
    type RawTarget = T;
    fn deref_raw(&self) -> *const Self::RawTarget {
        unsafe { self.inner.as_ptr().into_raw() }
    }
}

impl<C: RawCount, T: ?Sized + Debug> Debug for Strong<C, T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        self.deref().fmt(f)
    }
}
