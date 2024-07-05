use crate::raw_count::RawCount;
use std::cell::Cell;
use std::hint;
use std::hint::assert_unchecked;
use std::intrinsics::{abort, unlikely};

pub struct RawRc {
    strong: Cell<usize>,
    weak: Cell<usize>,
}

impl RawRc {
    unsafe fn strong<'a>(self: *const Self) -> &'a Cell<usize> {
        &(*self).strong
    }
    unsafe fn weak<'a>(self: *const Self) -> &'a Cell<usize> {
        &(*self).weak
    }
}

unsafe impl RawCount for RawRc {
    fn from_counts(strong: usize, weak: usize) -> Self {
        RawRc {
            strong: Cell::new(strong),
            weak: Cell::new(weak),
        }
    }

    #[inline]
    unsafe fn increment_strong_assume_non_zero(self: *const Self) {
        let strong = self.strong().get();
        unsafe {
            assert_unchecked(strong != 0);
        }
        let strong = strong.wrapping_add(1);
        self.strong().set(strong);
        if unlikely(strong == 0) {
            abort();
        }
    }

    #[inline]
    unsafe fn increment_strong_if_non_zero(self: *const Self) -> bool {
        let strong = self.strong().get();
        if strong == 0 {
            false
        } else {
            self.increment_strong_assume_non_zero();
            true
        }
    }

    #[inline]
    unsafe fn increment_strong_assume_zero(self: *const Self) {
        let strong = self.strong().get();
        unsafe {
            hint::assert_unchecked(strong == 0);
        }
        self.strong().set(1);
    }

    #[inline]
    unsafe fn decrement_strong(self: *const Self) -> bool {
        let strong = self.strong().get().unchecked_sub(1);
        self.strong().set(strong);
        strong == 0
    }

    #[inline]
    unsafe fn increment_weak(self: *const Self) {
        let weak = self.weak().get();
        unsafe {
            hint::assert_unchecked(weak != 0);
        }
        let weak = weak.wrapping_add(1);
        self.weak().set(weak);
        if unlikely(weak == 0) {
            abort();
        }
    }

    #[inline]
    unsafe fn decrement_weak(self: *const Self) -> bool {
        let weak = self.weak().get().unchecked_sub(1);
        self.weak().set(weak);
        weak == 0
    }
}
