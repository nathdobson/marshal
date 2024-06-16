use std::alloc::Layout;
use std::cell::UnsafeCell;
use std::fmt::Formatter;
use std::mem::{align_of_val_raw, size_of_val_raw};
use std::{fmt::Debug, marker::PhantomData, mem, sync};

use safe_once::sync::OnceLock;

use crate::arc_inner::allocate_arc_inner_raw_uninit;
use crate::{AsFlatRef, DerefRaw};

#[repr(transparent)]
pub struct ArcWeakRef<T: ?Sized> {
    phantom: PhantomData<*const ()>,
    inner: UnsafeCell<T>,
}

const FAKE_WEAK: OnceLock<&'static ()> = OnceLock::new();
const FAKE_WEAKS_SLICE: [OnceLock<&'static ()>; 64] = [FAKE_WEAK; 64];
pub static FAKE_WEAKS: [[OnceLock<&'static ()>; 64]; 64] = [FAKE_WEAKS_SLICE; 64];

fn fake_weak(layout: Layout) -> *const () {
    unsafe {
        let align_index = layout.align().ilog2() as usize;
        let size_index: usize;
        let new_size: usize;
        if let Some(size) = layout.size().checked_sub(1) {
            if let Some(log) = size.checked_ilog2() {
                size_index = log as usize + 2;
                new_size = 1 << size_index;
            } else {
                size_index = 1;
                new_size = 1;
            };
        } else {
            size_index = 0;
            new_size = 0;
        }
        *FAKE_WEAKS[align_index][size_index].get_or_init(|| {
            let layout = Layout::from_size_align(new_size, 1 << align_index).unwrap();
            let (arc_inner, value) = allocate_arc_inner_raw_uninit(layout);
            arc_inner.write_strong(0);
            arc_inner.write_weak(1);
            &*value
        })
    }
}

impl<T: ?Sized> AsFlatRef for sync::Weak<T> {
    type FlatRef = ArcWeakRef<T>;
    fn as_flat_ref(&self) -> &Self::FlatRef {
        unsafe {
            let ptr = self.as_ptr();
            if ptr as *const () as usize == usize::MAX {
                let layout =
                    Layout::from_size_align(size_of_val_raw(ptr), align_of_val_raw(ptr)).unwrap();
                let fake_weak = fake_weak(layout);
                &*(fake_weak.with_metadata_of(ptr) as *const ArcWeakRef<T>)
            } else {
                &*(self.as_ptr() as *const ArcWeakRef<T>)
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
        let foo = sync::Weak::<i32>::new().as_flat_ref().weak();
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
