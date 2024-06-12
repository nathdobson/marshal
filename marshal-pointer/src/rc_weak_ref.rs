use std::cell::UnsafeCell;
use std::fmt::Formatter;
use std::{fmt::Debug, marker::PhantomData, mem, rc};

use crate::AsFlatRef;

#[repr(transparent)]
pub struct RcWeakRef<T: ?Sized> {
    phantom: PhantomData<*const ()>,
    inner: UnsafeCell<T>,
}

impl<T: ?Sized> AsFlatRef for rc::Weak<T> {
    type FlatRef = RcWeakRef<T>;
    fn as_flat_ref(&self) -> &Self::FlatRef {
        unsafe { &*(self.as_ptr() as *const RcWeakRef<T>) }
    }
}

impl<T: ?Sized> RcWeakRef<T> {
    pub fn weak(&self) -> rc::Weak<T> {
        unsafe {
            let ptr = self as *const RcWeakRef<T> as *const T;
            let result = rc::Weak::from_raw(ptr);
            mem::forget(result.clone());
            result
        }
    }
}

impl<T: ?Sized> Debug for RcWeakRef<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RcWeak").finish_non_exhaustive()
    }
}

#[cfg(test)]
mod test {
    use crate::AsFlatRef;
    use std::rc::Rc;

    #[test]
    fn test() {
        let x = Rc::new(123);
        Rc::downgrade(&x).as_flat_ref().weak();
    }
}
