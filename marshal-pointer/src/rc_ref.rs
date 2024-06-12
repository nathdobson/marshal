use std::{fmt::Debug, hash::Hash, marker::PhantomData, mem, ops::Deref, rc::Rc};

use crate::PtrRef;

#[repr(transparent)]
pub struct RcRef<T: ?Sized> {
    phantom: PhantomData<*const ()>,
    inner: T,
}

impl<T: ?Sized> PtrRef for Rc<T> {
    type PtrTarget = RcRef<T>;

    fn ptr_ref(&self) -> &Self::PtrTarget {
        unsafe { mem::transmute::<&T, &RcRef<T>>(&**self) }
    }
}

unsafe impl<T: ?Sized> Sync for RcRef<T> where T: Sync + Send {}
unsafe impl<T: ?Sized> Send for RcRef<T> where T: Sync + Send {}

impl<T: ?Sized> RcRef<T> {
    pub fn arc(&self) -> Rc<T> {
        unsafe {
            Rc::<Self>::increment_strong_count(self);
            Rc::<T>::from_raw(self as *const RcRef<T> as *const T)
        }
    }
}

impl<T: ?Sized> Deref for RcRef<T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}
