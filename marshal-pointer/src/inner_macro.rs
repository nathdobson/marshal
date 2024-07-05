use std::{mem, rc, sync};
use std::alloc::{Global, handle_alloc_error, Layout};
use std::alloc::Allocator;
use std::cell::Cell;
use std::intrinsics::{abort, unlikely};
use std::mem::{align_of_val_raw, size_of_val_raw};
use std::rc::Rc;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

macro_rules! inner {
    ($inner:ident, $param:ident, $count:ty, $strong:ty, $weak:ty) => {
        #[repr(C)]
        pub struct $inner<$param: ?Sized> {
            strong: $count,
            weak: $count,
            inner: $param,
        }

        impl<$param: ?Sized> $inner<$param> {
            pub fn allocate_uninit() -> *mut Self
            where
                $param: Sized,
            {
                let layout = Layout::new::<Self>();
                Global
                    .allocate(layout)
                    .unwrap_or_else(|_| handle_alloc_error(layout))
                    .cast::<Self>()
                    .as_ptr()
            }
            unsafe fn weak_count_mut(self: *mut Self) -> *mut $count {
                &raw mut (*self).weak
            }
            unsafe fn strong_count_mut(self: *mut Self) -> *mut $count {
                &raw mut (*self).strong
            }
            pub unsafe fn inner_mut(self: *mut Self) -> *mut $param {
                &raw mut (*self).inner
            }
            unsafe fn weak_count(self: *const Self) -> *const $count {
                &raw const (*self).weak
            }
            unsafe fn strong_count(self: *const Self) -> *const $count {
                &raw const (*self).strong
            }
            pub unsafe fn inner(self: *const Self) -> *const $param {
                &raw const (*self).inner
            }
            pub unsafe fn write_strong(self: *mut Self, value: usize) {
                self.strong_count_mut().write(<$count>::from(value))
            }
            pub unsafe fn write_weak(self: *mut Self, value: usize) {
                self.weak_count_mut().write(<$count>::from(value))
            }
            pub unsafe fn write_inner(self: *mut Self, value: T)
            where
                T: Sized,
            {
                self.inner_mut().write(value)
            }

            pub unsafe fn drop_inner(self: *mut Self) {
                self.inner_mut().drop_in_place();
            }
            pub unsafe fn into_weak(self: *const Self) -> $weak {
                mem::transmute(self)
            }
            pub unsafe fn into_strong(self: *const Self) -> $strong {
                mem::transmute(self)
            }
            pub unsafe fn inner_deref<'a>(self: *const Self) -> &'a $param {
                &*self.inner()
            }
            pub unsafe fn inner_deref_mut<'a>(self: *mut Self) -> &'a mut $param {
                &mut *self.inner_mut()
            }
            pub unsafe fn from_raw(inner: *const $param) -> *const Self {
                let offset = Layout::new::<()>()
                    .extend(Layout::new::<$count>())
                    .unwrap()
                    .0
                    .extend(Layout::new::<$count>())
                    .unwrap()
                    .0
                    .extend(
                        Layout::from_size_align(size_of_val_raw(inner), align_of_val_raw(inner))
                            .unwrap(),
                    )
                    .unwrap()
                    .1;
                inner.byte_sub(offset) as *const Self
            }
            pub unsafe fn from_strong(arc: &$strong) -> *const Self {
                mem::transmute_copy::<$strong, *const Self>(arc)
            }
        }
    };
}

inner!(ArcInner, T, AtomicUsize, Arc<T>, sync::Weak<T>);
inner!(RcInner, T, Cell<usize>, Rc<T>, rc::Weak<T>);

impl<T: ?Sized> ArcInner<T> {
    pub unsafe fn increment_weak(self: *const Self) {
        let count = (*self.weak_count()).fetch_add(1, Ordering::Relaxed);
        if count > ARC_MAX_REFCOUNT {
            abort();
        }
    }
    pub unsafe fn increment_strong(self: *const Self) {
        let count = (*self.strong_count()).fetch_add(1, Ordering::Relaxed);
        if count > ARC_MAX_REFCOUNT {
            abort();
        }
    }
}

impl<T:?Sized> RcInner<T>{
    pub unsafe fn increment_weak(self: *const Self) {
        let weak_ref = &*self.weak_count();
        let weak = weak_ref.get();
        let weak = weak.wrapping_add(1);
        weak_ref.set(weak);
        if unlikely(weak == 0) {
            abort();
        }
    }
    pub unsafe fn increment_strong(self: *const Self) {
        let strong_ref = &*self.strong_count();
        let strong = strong_ref.get();
        let strong = strong.wrapping_add(1);
        strong_ref.set(strong);
        if unlikely(strong == 0) {
            abort();
        }
    }
}

const ARC_MAX_REFCOUNT: usize = (isize::MAX) as usize;
