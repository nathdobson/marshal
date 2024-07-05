use crate::inner::Inner;
use crate::raw_any::{DerefRaw, DowncastError, DowncastRef, RawAny};
use crate::raw_count::RawCount;
use crate::strong::Strong;
use crate::strong_ref::StrongRef;
use crate::weak::Weak;
use std::cell::UnsafeCell;
use std::marker::PhantomData;
use std::mem::ManuallyDrop;

pub struct WeakRef<C: RawCount, T: ?Sized> {
    phantom: PhantomData<C>,
    value: UnsafeCell<T>,
}

impl<C: RawCount, T: ?Sized> WeakRef<C, T> {
    pub fn strong(&self) -> Option<Strong<C, T>> {
        unsafe {
            let raw = Inner::<C, T>::from_raw(&self.value as *const _ as *const T);
            if raw.count_raw().increment_strong_if_non_zero() {
                Some(Strong::from_inner(raw))
            } else {
                None
            }
        }
    }
    pub fn weak(&self) -> Weak<C, T> {
        unsafe {
            let raw = Inner::<C, T>::from_raw(&self.value as *const _ as *const T);
            raw.count_raw().increment_weak();
            Weak::from_inner(raw)
        }
    }
}

impl<C: RawCount, T: ?Sized> DerefRaw for WeakRef<C, T> {
    type RawTarget = T;
    fn deref_raw(&self) -> *const Self::RawTarget {
        &self.value as *const _ as *const T
    }
}

impl<C: RawCount, T: 'static> DowncastRef<WeakRef<C, T>> for WeakRef<C, dyn RawAny> {
    fn downcast_ref(&self) -> Result<&WeakRef<C, T>, DowncastError<()>> {
        unsafe {
            let this = UnsafeCell::raw_get((&self.value) as *const UnsafeCell<dyn RawAny>);
            let this = this as *const dyn RawAny;
            this.downcast_check::<T>()?;
            Ok(&*(self as *const WeakRef<C, dyn RawAny> as *const WeakRef<C, T>))
        }
    }
}
