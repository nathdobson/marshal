use crate::arc_ref::ArcRef;
use crate::global_uninit::global_uninit_for_ptr;
use crate::{AsFlatRef, DerefRaw, DowncastRef, RawAny};
use std::any::TypeId;
use std::cell::UnsafeCell;
use std::fmt::Formatter;
use std::sync::Arc;
use std::{fmt::Debug, marker::PhantomData, mem, sync};

#[repr(transparent)]
pub struct ArcWeakRef<T: ?Sized> {
    phantom: PhantomData<*const ()>,
    inner: UnsafeCell<T>,
}

impl<T: ?Sized> AsFlatRef for sync::Weak<T> {
    type FlatRef = ArcWeakRef<T>;
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

unsafe impl<T: ?Sized> Sync for ArcWeakRef<T> where T: Sync + Send {}
unsafe impl<T: ?Sized> Send for ArcWeakRef<T> where T: Sync + Send {}

impl<T: ?Sized> ArcWeakRef<T> {
    pub fn weak(&self) -> sync::Weak<T> {
        unsafe {
            let ptr = self as *const ArcWeakRef<T> as *const T;
            if ptr as *const () == global_uninit_for_ptr::<T>(ptr) as *const () {
                return sync::Weak::from_raw(
                    (std::ptr::without_provenance::<()>(usize::MAX) as *const ())
                        .with_metadata_of(ptr),
                );
            }
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

impl<T: ?Sized> DerefRaw for sync::Weak<T> {
    type RawTarget = T;
    fn deref_raw(&self) -> *const Self::RawTarget {
        self.as_ptr()
    }
}

impl<T: ?Sized> DerefRaw for ArcWeakRef<T> {
    type RawTarget = T;
    fn deref_raw(&self) -> *const Self::RawTarget {
        self as *const ArcWeakRef<T> as *const T
    }
}

impl<T: 'static> DowncastRef<ArcWeakRef<T>> for ArcWeakRef<dyn RawAny> {
    fn downcast_ref(&self) -> Option<&ArcWeakRef<T>> {
        unsafe {
            if self.deref_raw().raw_type_id() == TypeId::of::<T>() {
                Some(&*(self as *const ArcWeakRef<dyn RawAny> as *const ArcWeakRef<T>))
            } else {
                None
            }
        }
    }
}

impl<T> AsRef<ArcWeakRef<T>> for sync::Weak<T> {
    fn as_ref(&self) -> &ArcWeakRef<T> {
        self.as_flat_ref()
    }
}

#[cfg(test)]
mod test {
    use std::sync;
    use std::sync::Arc;

    use crate::AsFlatRef;

    #[test]
    fn test() {
        let x = Arc::new(123);
        Arc::downgrade(&x).as_flat_ref().weak();
    }

    #[test]
    fn test2() {
        let _ = sync::Weak::<i32>::new().as_flat_ref().weak();
    }

    #[test]
    fn test_fake_weak() {
        fn get_fake<T>() -> sync::Weak<T> {
            sync::Weak::new().as_flat_ref().weak()
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
