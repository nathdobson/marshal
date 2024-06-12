use std::marker::PhantomData;
use std::ops::Deref;
use std::rc::Rc;
use crate::AsFlatRef;

#[repr(transparent)]
pub struct RcRef<T: ?Sized> {
    phantom: PhantomData<*const ()>,
    inner: T,
}

impl<T: ?Sized> AsFlatRef for Rc<T> {
    type FlatRef = RcRef<T>;

    fn as_flat_ref(&self) -> &Self::FlatRef {
        unsafe { &*((&**self) as *const T as *const RcRef<T>) }
    }
}

impl<T: ?Sized> RcRef<T> {
    pub fn rc(&self) -> Rc<T> {
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

#[test]
fn test() {
    Rc::new(123).as_flat_ref().rc();
}
