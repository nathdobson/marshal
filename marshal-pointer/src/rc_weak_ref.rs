use std::cell::UnsafeCell;
use std::fmt::Formatter;
use std::{fmt::Debug, marker::PhantomData, mem, rc};
use std::any::TypeId;

use crate::{AsFlatRef, DerefRaw, DowncastRef, RawAny};

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

impl<T: ?Sized> DerefRaw for rc::Weak<T> {
    type RawTarget = T;
    fn deref_raw(&self) -> *const Self::RawTarget {
        self.as_ptr()
    }
}

impl<T: ?Sized> DerefRaw for RcWeakRef<T> {
    type RawTarget = T;
    fn deref_raw(&self) -> *const Self::RawTarget {
        self as *const RcWeakRef<T> as *const T
    }
}

impl<T: 'static> DowncastRef<RcWeakRef<T>> for RcWeakRef<dyn RawAny> {
    fn downcast_ref(&self) -> Option<&RcWeakRef<T>> {
        unsafe {
            if self.deref_raw().raw_type_id() == TypeId::of::<T>() {
                Some(&*(self as *const RcWeakRef<dyn RawAny> as *const RcWeakRef<T>))
            } else {
                None
            }
        }
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
