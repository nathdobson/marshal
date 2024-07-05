use crate::inner::Inner;
use crate::raw_count::RawCount;
use crate::strong::Strong;
use crate::weak::Weak;
use std::marker::PhantomData;
use std::ops::Deref;

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
