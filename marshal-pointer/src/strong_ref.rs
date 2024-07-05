use std::marker::PhantomData;
use std::ops::Deref;

use crate::inner::Inner;
use crate::raw_any::{DerefRaw, DowncastError, DowncastRef, RawAny};
use crate::raw_count::RawCount;
use crate::strong::Strong;
use crate::weak::Weak;

#[repr(transparent)]
pub struct StrongRef<C: RawCount, T: ?Sized> {
    phantom: PhantomData<C>,
    value: T,
}

impl<C: RawCount, T: ?Sized> StrongRef<C, T> {
    pub fn strong(&self) -> Strong<C, T> {
        unsafe {
            let raw = Inner::<C, T>::from_raw(&self.value);
            raw.count_raw().increment_strong_assume_non_zero();
            Strong::from_inner(raw)
        }
    }
    pub fn weak(&self) -> Weak<C, T> {
        unsafe {
            let raw = Inner::<C, T>::from_raw(&self.value);
            raw.count_raw().increment_weak();
            Weak::from_inner(raw)
        }
    }
}

impl<C: RawCount, T: ?Sized> Deref for StrongRef<C, T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

impl<C: RawCount, T: ?Sized> DerefRaw for StrongRef<C, T> {
    type RawTarget = T;
    fn deref_raw(&self) -> *const Self::RawTarget {
        &self.value
    }
}

impl<C: RawCount, T: 'static> DowncastRef<StrongRef<C, T>> for StrongRef<C, dyn RawAny> {
    fn downcast_ref(&self) -> Result<&StrongRef<C, T>, DowncastError<()>> {
        unsafe {
            (&self.value as *const dyn RawAny).downcast_check::<T>()?;
            Ok(&*(self as *const StrongRef<C, dyn RawAny> as *const StrongRef<C, T>))
        }
    }
}
