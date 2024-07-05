use crate::raw_count::RawCount;
use std::process::abort;
use std::sync::atomic;
use std::sync::atomic::{AtomicUsize, Ordering};

pub struct RawArc {
    strong: AtomicUsize,
    weak: AtomicUsize,
}

impl RawArc {
    unsafe fn strong<'a>(self: *const Self) -> &'a AtomicUsize {
        &(*self).strong
    }
    unsafe fn weak<'a>(self: *const Self) -> &'a AtomicUsize {
        &(*self).weak
    }
}

unsafe impl RawCount for RawArc {
    #[inline]
    fn from_counts(strong: usize, weak: usize) -> Self {
        RawArc {
            strong: AtomicUsize::new(strong),
            weak: AtomicUsize::new(weak),
        }
    }

    #[inline]
    unsafe fn increment_strong_assume_non_zero(self: *const Self) {
        let old = self.strong().fetch_add(1, Ordering::Relaxed);
        if old > MAX_REFCOUNT {
            abort();
        }
    }

    #[inline]
    unsafe fn increment_strong_if_non_zero(self: *const Self) -> bool {
        #[inline]
        fn checked_increment(n: usize) -> Option<usize> {
            if n == 0 {
                return None;
            }
            Some(n + 1)
        }

        if let Ok(x) =
            self.strong()
                .fetch_update(Ordering::Acquire, Ordering::Relaxed, checked_increment)
        {
            if x > MAX_REFCOUNT {
                abort()
            }
            true
        } else {
            false
        }
    }

    #[inline]
    unsafe fn increment_strong_assume_zero(self: *const Self) {
        self.strong().store(1, Ordering::Release);
    }

    #[inline]
    unsafe fn decrement_strong(self: *const Self) -> bool {
        if self.strong().fetch_sub(1, Ordering::Release) != 1 {
            return false;
        }
        atomic::fence(Ordering::Acquire);
        true
    }

    #[inline]
    unsafe fn increment_weak(self: *const Self) {
        let old = self.weak().fetch_add(1, Ordering::Relaxed);
        if old > MAX_REFCOUNT {
            abort();
        }
    }

    #[inline]
    unsafe fn decrement_weak(self: *const Self) -> bool {
        if self.weak().fetch_sub(1, Ordering::Release) == 1 {
            atomic::fence(Ordering::Acquire);
            true
        } else {
            false
        }
    }
}

const MAX_REFCOUNT: usize = isize::MAX as usize;
