use std::alloc::{handle_alloc_error, Allocator, Global, Layout};
use std::cell::Cell;
use std::intrinsics::{abort, unlikely};
use std::mem::{align_of_val_raw, size_of_val_raw};
use std::rc::Rc;
use std::{hint, mem, rc};

#[repr(C)]
pub struct RcInner<T: ?Sized> {
    strong: Cell<usize>,
    weak: Cell<usize>,
    inner: T,
}

impl<T: ?Sized> RcInner<T> {
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
    unsafe fn weak_count_mut(self: *mut Self) -> *mut Cell<usize> {
        &raw mut (*self).weak
    }
    unsafe fn strong_count_mut(self: *mut Self) -> *mut Cell<usize> {
        &raw mut (*self).strong
    }
    pub unsafe fn inner_mut(self: *mut Self) -> *mut T {
        &raw mut (*self).inner
    }
    unsafe fn weak_count(self: *const Self) -> *const Cell<usize> {
        &raw const (*self).weak
    }
    unsafe fn strong_count(self: *const Self) -> *const Cell<usize> {
        &raw const (*self).strong
    }
    pub unsafe fn inner(self: *const Self) -> *const T {
        &raw const (*self).inner
    }
    pub unsafe fn write_strong(self: *mut Self, value: usize) {
        self.strong_count_mut().write(Cell::new(value))
    }
    pub unsafe fn write_weak(self: *mut Self, value: usize) {
        self.weak_count_mut().write(Cell::new(value))
    }
    pub unsafe fn write_inner(self: *mut Self, value: T)
    where
        T: Sized,
    {
        self.inner_mut().write(value)
    }

    pub unsafe fn increment_weak(self: *const Self) {
        let weak_ref = &*self.weak_count();
        let weak = weak_ref.get();
        unsafe {
            hint::assert_unchecked(weak != 0);
        }
        let weak = weak.wrapping_add(1);
        weak_ref.set(weak);
        if unlikely(weak == 0) {
            abort();
        }
    }
    pub unsafe fn increment_strong(self: *const Self) {
        let strong_ref = &*self.strong_count();
        let strong = strong_ref.get();
        unsafe {
            hint::assert_unchecked(strong != 0);
        }
        let strong = strong.wrapping_add(1);
        strong_ref.set(strong);
        if unlikely(strong == 0) {
            abort();
        }
    }
    pub unsafe fn drop_inner(self: *mut Self) {
        self.inner_mut().drop_in_place();
    }
    pub unsafe fn into_weak(self: *const Self) -> rc::Weak<T> {
        mem::transmute(self)
    }
    pub unsafe fn into_rc(self: *const Self) -> Rc<T> {
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
            .extend(Layout::new::<Cell<usize>>())
            .unwrap()
            .0
            .extend(Layout::new::<Cell<usize>>())
            .unwrap()
            .0
            .extend(
                Layout::from_size_align(size_of_val_raw(inner), align_of_val_raw(inner)).unwrap(),
            )
            .unwrap()
            .1;
        inner.byte_sub(offset) as *const Self
    }
    pub unsafe fn from_rc(arc: &Rc<T>) -> *const Self {
        mem::transmute_copy::<Rc<T>, *const RcInner<T>>(arc)
    }
}

const MAX_REFCOUNT: usize = (isize::MAX) as usize;
