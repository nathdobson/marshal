use std::sync::atomic::{AtomicU64, Ordering};

#[derive(Copy, Clone, Eq, Ord, PartialEq, PartialOrd)]
pub struct Version {
    global: u64,
    local: u64,
}

pub static GLOBAL: AtomicU64 = AtomicU64::new(0);

impl Version {
    pub fn new() -> Self {
        Version {
            global: GLOBAL.fetch_add(1, Ordering::Relaxed),
            local: 0,
        }
    }
    pub fn next(&mut self) {
        self.local += 1;
    }
}
