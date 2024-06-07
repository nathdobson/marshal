use lock_api::{Mutex, RawMutex};
use safe_once::api::raw::RawFused;
use safe_once::cell::{OnceCell, RawFusedCell};
use safe_once_map::util::index_arena::IndexArena;
use std::borrow::{Borrow, Cow};
use std::cell::Cell;
use std::collections::HashMap;
use std::hash::{BuildHasher, Hash};
use std::ops::Index;

pub struct StableCellVec<T> {
    arena: IndexArena<RawFusedCell, OnceCell<T>>,
    next: Cell<usize>,
}

impl<T> StableCellVec<T> {
    pub fn new() -> Self {
        StableCellVec {
            arena: IndexArena::new(),
            next: Cell::new(0),
        }
    }
    pub fn get(&self, index: usize) -> Option<&T> {
        Some(self.arena.try_get(index)?.get()?)
    }
    pub fn push(&self, value: T) -> usize {
        let index = self.next.get();
        self.next.set(index + 1);
        self.arena.get_or_init(index).set(value).ok().unwrap();
        index
    }
}

impl<T> Default for StableCellVec<T> {
    fn default() -> Self {
        StableCellVec::new()
    }
}
