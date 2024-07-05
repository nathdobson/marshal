use crate::inner::Inner;
use crate::raw_count::RawCount;
use crate::strong::Strong;
use crate::weak::Weak;
use std::marker::PhantomData;

pub struct WeakRef<C: RawCount, T: ?Sized> {
    phantom: PhantomData<C>,
    value: T,
}

impl<C: RawCount, T: ?Sized> WeakRef<C, T> {
    pub fn strong(&self) -> Option<Strong<C, T>> {
        unsafe {
            let raw = Inner::<C, T>::from_raw(&self.value);
            if raw.count_raw().increment_strong_if_non_zero() {
                Some(Strong::from_inner(raw))
            } else {
                None
            }
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
