use std::{fmt::Debug, marker::PhantomData, mem, rc};
use std::any::TypeId;
use std::cell::UnsafeCell;
use std::fmt::Formatter;

use crate::{AsFlatRef, DerefRaw, DowncastRef, RawAny};
use crate::global_uninit::global_uninit_for_ptr;

#[repr(transparent)]
pub struct RcWeakRef<T: ?Sized> {
    phantom: PhantomData<*const ()>,
    inner: UnsafeCell<T>,
}

impl<T: ?Sized> AsFlatRef for rc::Weak<T> {
    type FlatRef = RcWeakRef<T>;
    fn as_flat_ref(&self) -> &Self::FlatRef {
        unsafe {
            let ptr = self.as_ptr();
            if ptr as *const () as usize == usize::MAX {
                &*(global_uninit_for_ptr::<T>(ptr) as *const Self::FlatRef)
            } else {
                &*(self.as_ptr() as *const Self::FlatRef)
            }
        }
    }
}

impl<T: ?Sized> RcWeakRef<T> {
    pub fn weak(&self) -> rc::Weak<T> {
        unsafe {
            let ptr = self as *const RcWeakRef<T> as *const T;
            if ptr as *const () == global_uninit_for_ptr::<T>(ptr) as *const () {
                return rc::Weak::from_raw(
                    (std::ptr::without_provenance::<()>(usize::MAX) as *const ())
                        .with_metadata_of(ptr),
                );
            }
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

impl<T> AsRef<RcWeakRef<T>> for rc::Weak<T> {
    fn as_ref(&self) -> &RcWeakRef<T> {
        self.as_flat_ref()
    }
}

#[cfg(test)]
mod test {
    use std::rc;
    use std::rc::Rc;

    use crate::AsFlatRef;

    #[test]
    fn test() {
        let x = Rc::new(123);
        Rc::downgrade(&x).as_flat_ref().weak();
    }

    #[test]
    fn test_fake_weak() {
        fn get_fake<T>() -> rc::Weak<T> {
            rc::Weak::new().as_flat_ref().weak()
        }
        struct Foo;
        get_fake::<Foo>();

        #[repr(align(2))]
        struct Align2<T>(T);
        #[repr(align(4))]
        struct Align4<T>(T);
        #[repr(align(8))]
        struct Align8<T>(T);
        #[repr(align(8192))]
        struct Align8192<T>(T);
        get_fake::<Align2<[u8; 0]>>();
        get_fake::<Align2<[u8; 1]>>();
        get_fake::<Align2<[u8; 2]>>();
        get_fake::<Align2<[u8; 3]>>();
        get_fake::<Align2<[u8; 4]>>();
        get_fake::<Align2<[u8; 5]>>();
        get_fake::<Align2<[u8; 6]>>();
        get_fake::<Align2<[u8; 7]>>();
        get_fake::<Align2<[u8; 8]>>();
        get_fake::<Align2<[u8; 9]>>();

        get_fake::<Align4<[u8; 0]>>();
        get_fake::<Align4<[u8; 1]>>();
        get_fake::<Align4<[u8; 2]>>();
        get_fake::<Align4<[u8; 3]>>();
        get_fake::<Align4<[u8; 4]>>();
        get_fake::<Align4<[u8; 5]>>();
        get_fake::<Align4<[u8; 6]>>();
        get_fake::<Align4<[u8; 7]>>();
        get_fake::<Align4<[u8; 8]>>();
        get_fake::<Align4<[u8; 9]>>();

        get_fake::<Align8<[u8; 0]>>();
        get_fake::<Align8<[u8; 1]>>();
        get_fake::<Align8<[u8; 2]>>();
        get_fake::<Align8<[u8; 3]>>();
        get_fake::<Align8<[u8; 4]>>();
        get_fake::<Align8<[u8; 5]>>();
        get_fake::<Align8<[u8; 6]>>();
        get_fake::<Align8<[u8; 7]>>();
        get_fake::<Align8<[u8; 8]>>();
        get_fake::<Align8<[u8; 9]>>();

        get_fake::<Align8192<[u8; 0]>>();
        get_fake::<Align8192<[u8; 1]>>();
        get_fake::<Align8192<[u8; 2]>>();
        get_fake::<Align8192<[u8; 3]>>();
        get_fake::<Align8192<[u8; 4]>>();
        get_fake::<Align8192<[u8; 5]>>();
        get_fake::<Align8192<[u8; 6]>>();
        get_fake::<Align8192<[u8; 7]>>();
        get_fake::<Align8192<[u8; 8]>>();
        get_fake::<Align8192<[u8; 9]>>();
    }
}
