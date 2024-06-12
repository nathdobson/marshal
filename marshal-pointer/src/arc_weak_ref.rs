use std::cell::UnsafeCell;
use std::fmt::Formatter;
use std::{fmt::Debug, marker::PhantomData, mem, sync};

use crate::AsFlatRef;

#[repr(transparent)]
pub struct ArcWeakRef<T: ?Sized> {
    phantom: PhantomData<*const ()>,
    inner: UnsafeCell<T>,
}

impl<T: ?Sized> AsFlatRef for sync::Weak<T> {
    type FlatRef = ArcWeakRef<T>;
    fn as_flat_ref(&self) -> &Self::FlatRef {
        unsafe { &*(self.as_ptr() as *const ArcWeakRef<T>) }
    }
}

unsafe impl<T: ?Sized> Sync for ArcWeakRef<T> where T: Sync + Send {}
unsafe impl<T: ?Sized> Send for ArcWeakRef<T> where T: Sync + Send {}

impl<T: ?Sized> ArcWeakRef<T> {
    pub fn weak(&self) -> sync::Weak<T> {
        unsafe {
            let ptr = self as *const ArcWeakRef<T> as *const T;
            let result = sync::Weak::from_raw(ptr);
            mem::forget(result.clone());
            result
        }
    }
}

impl<T: ?Sized> Debug for ArcWeakRef<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ArcWeak").finish_non_exhaustive()
    }
}

#[cfg(test)]
mod test {
    use crate::AsFlatRef;
    use std::sync::Arc;

    #[test]
    fn test() {
        let x = Arc::new(123);
        Arc::downgrade(&x).as_flat_ref().weak();
    }
}
