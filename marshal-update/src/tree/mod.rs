use std::any::Any;
use std::fmt::{Debug, Display, Formatter};
use std::marker::Unsize;
use std::ops::{Deref, DerefMut};
use std::sync::Arc;

use atomic_refcell::{AtomicRef, AtomicRefCell, AtomicRefMut};
use by_address::ByThinAddress;
use safe_once::sync::OnceLock;

use crate::tree::id::ForestId;
use crate::tree::ser::SerializeQueue;

pub mod bin;
pub mod de;
mod id;
pub mod json;
pub mod ser;

pub struct Forest {
    id: ForestId,
}

impl Forest {
    pub fn new() -> Self {
        Forest {
            id: ForestId::new(),
        }
    }
    pub fn add<T: Sync + Send>(&mut self, value: T) -> Arc<Tree<T>> {
        Arc::new(Tree::new(value, self.id))
    }
}

#[derive(Debug)]
pub enum TreeError {
    MissingId,
}

impl Display for TreeError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        Debug::fmt(self, f)
    }
}

impl std::error::Error for TreeError {}

pub struct TreeState<T: ?Sized> {
    stream: Option<Box<dyn Sync + Send + Any>>,
    value: T,
}

pub struct Tree<T: ?Sized> {
    _forest_id: ForestId,
    serialize_queue: OnceLock<Arc<SerializeQueue>>,
    state: AtomicRefCell<TreeState<T>>,
}

pub struct TreeReadGuard<'a, T: ?Sized> {
    state: AtomicRef<'a, TreeState<T>>,
}

pub struct TreeWriteGuard<'a, T: ?Sized> {
    state: AtomicRefMut<'a, TreeState<T>>,
}

impl<'a, T: ?Sized> Deref for TreeReadGuard<'a, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.state.value
    }
}

impl<'a, T: ?Sized> Deref for TreeWriteGuard<'a, T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        &self.state.value
    }
}

impl<'a, T: ?Sized> DerefMut for TreeWriteGuard<'a, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.state.value
    }
}

impl<T: Sync + Send + ?Sized> Tree<T> {
    pub fn new(value: T, forest_id: ForestId) -> Self
    where
        T: Sized,
    {
        Tree {
            _forest_id: forest_id,
            serialize_queue: OnceLock::new(),
            state: AtomicRefCell::new(TreeState {
                stream: None,
                value,
            }),
        }
    }
    pub fn read(&self) -> TreeReadGuard<T> {
        TreeReadGuard {
            state: self.state.borrow(),
        }
    }
    pub fn write<'a>(self: &'a Arc<Self>) -> TreeWriteGuard<'a, T>
    where
        T: Unsize<dyn Sync + Send + Any>,
    {
        if let Some(forest) = self.serialize_queue.get() {
            forest.queue.lock().insert(ByThinAddress(self.clone()));
        }
        TreeWriteGuard {
            state: self.state.borrow_mut(),
        }
    }
}
