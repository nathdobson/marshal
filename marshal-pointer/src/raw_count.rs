pub unsafe trait RawCount {
    fn from_counts(strong: usize, weak: usize) -> Self;
    unsafe fn increment_strong_assume_non_zero(self: *const Self);
    unsafe fn increment_strong_if_non_zero(self: *const Self) -> bool;
    unsafe fn increment_strong_assume_zero(self: *const Self);
    unsafe fn decrement_strong(self: *const Self) -> bool;
    unsafe fn increment_weak(self: *const Self);
    unsafe fn decrement_weak(self: *const Self) -> bool;
}
