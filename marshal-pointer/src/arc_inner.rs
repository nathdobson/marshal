use std::{mem, sync};
use std::alloc::{Allocator, Global, handle_alloc_error, Layout};
use std::intrinsics::abort;
use std::mem::{align_of_val_raw, size_of_val_raw};
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

#[repr(C)]
pub struct ArcInner<T: ?Sized> {
    strong: AtomicUsize,
    weak: AtomicUsize,
    inner: T,
}


impl<T: ?Sized> ArcInner<T> {
    pub fn allocate_uninit() -> *mut Self
    where
        T: Sized,
    {
        let layout = Layout::new::<Self>();
        Global
            .allocate(layout)
            .unwrap_or_else(|_| handle_alloc_error(layout))
            .cast::<Self>()
            .as_ptr()
    }
    unsafe fn weak_count_mut(self: *mut Self) -> *mut AtomicUsize {
        &raw mut (*self).weak
    }
    unsafe fn strong_count_mut(self: *mut Self) -> *mut AtomicUsize {
        &raw mut (*self).strong
    }
    pub unsafe fn inner_mut(self: *mut Self) -> *mut T {
        &raw mut (*self).inner
    }
    unsafe fn weak_count(self: *const Self) -> *const AtomicUsize {
        &raw const (*self).weak
    }
    unsafe fn strong_count(self: *const Self) -> *const AtomicUsize {
        &raw const (*self).strong
    }
    pub unsafe fn inner(self: *const Self) -> *const T {
        &raw const (*self).inner
    }
    pub unsafe fn write_strong(self: *mut Self, value: usize) {
        self.strong_count_mut().write(AtomicUsize::new(value))
    }
    pub unsafe fn write_weak(self: *mut Self, value: usize) {
        self.weak_count_mut().write(AtomicUsize::new(value))
    }
    pub unsafe fn write_inner(self: *mut Self, value: T)
    where
        T: Sized,
    {
        self.inner_mut().write(value)
    }

    pub unsafe fn increment_weak(self: *const Self) {
        let count = (*self.weak_count()).fetch_add(1, Ordering::Relaxed);
        if count > MAX_REFCOUNT {
            abort();
        }
    }
    pub unsafe fn increment_strong(self: *const Self) {
        let count = (*self.strong_count()).fetch_add(1, Ordering::Relaxed);
        if count > MAX_REFCOUNT {
            abort();
        }
    }
    pub unsafe fn drop_inner(self: *mut Self) {
        self.inner_mut().drop_in_place();
    }
    pub unsafe fn into_weak(self: *const Self) -> sync::Weak<T> {
        mem::transmute(self)
    }
    pub unsafe fn into_arc(self: *const Self) -> Arc<T> {
        mem::transmute(self)
    }
    pub unsafe fn inner_deref<'a>(self: *const Self) -> &'a T {
        &*self.inner()
    }
    pub unsafe fn inner_deref_mut<'a>(self: *mut Self) -> &'a mut T {
        &mut *self.inner_mut()
    }
    pub unsafe fn from_raw(inner: *const T) -> *const Self {
        let offset = Layout::new::<()>()
            .extend(Layout::new::<AtomicUsize>())
            .unwrap()
            .0
            .extend(Layout::new::<AtomicUsize>())
            .unwrap()
            .0
            .extend(
                Layout::from_size_align(size_of_val_raw(inner), align_of_val_raw(inner)).unwrap(),
            )
            .unwrap()
            .1;
        inner.byte_sub(offset) as *const Self
    }
    pub unsafe fn from_arc(arc: &Arc<T>) -> *const Self {
        mem::transmute_copy::<Arc<T>, *const ArcInner<T>>(arc)
    }
}

const MAX_REFCOUNT: usize = (isize::MAX) as usize;
