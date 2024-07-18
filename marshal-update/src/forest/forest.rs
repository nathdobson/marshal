use std::any::Any;
use std::collections::HashSet;
use std::fmt::{Debug, Formatter};
use std::marker::Unsize;
use std::sync::Arc;

use atomic_refcell::AtomicRefCell;
use by_address::ByAddress;
use tokenlock::{IcToken, IcTokenId, IcTokenLock};

use marshal_pointer::Arcf;

use crate::forest::de::ForestDeserializerTable;
use crate::forest::ser::ForestSerializerTable;
use crate::ser::set_channel::{SetPublisher, SetSubscriber};

#[derive(Copy, Clone)]
pub struct ForestId {
    token: IcTokenId,
}

pub struct Forest {
    token: IcToken,
    publisher: SetPublisher<HashSet<ByAddress<Arcf<Tree<dyn Sync + Send + Any>>>>>,
    serializers: AtomicRefCell<ForestSerializerTable>,
}

pub struct ForestRoot<T: ?Sized> {
    forest: Forest,
    deserializers: ForestDeserializerTable,
    root: T,
}

pub struct Tree<T: ?Sized> {
    inner: IcTokenLock<T>,
}

impl<T> ForestRoot<T> {
    pub fn new(forest: Forest, root: T) -> Self {
        let id = forest.id();
        Self::new_raw(forest, ForestDeserializerTable::new(id), root)
    }
    pub(super) fn new_raw(forest: Forest, deserializers: ForestDeserializerTable, root: T) -> Self {
        ForestRoot {
            forest,
            deserializers,
            root,
        }
    }
    pub fn forest(&self) -> &Forest {
        &self.forest
    }
    pub fn forest_mut(&mut self) -> &mut Forest {
        &mut self.forest
    }
    pub fn root(&self) -> &T {
        &self.root
    }
    pub fn root_mut(&mut self) -> &mut T {
        &mut self.root
    }
    pub(super) fn view_mut_internal(
        &mut self,
    ) -> (&mut T, &mut Forest, &mut ForestDeserializerTable) {
        (&mut self.root, &mut self.forest, &mut self.deserializers)
    }
    pub fn view_mut(&mut self) -> (&mut T, &mut Forest) {
        (&mut self.root, &mut self.forest)
    }
}

impl Forest {
    pub fn new() -> Self {
        Forest {
            token: IcToken::new(),
            publisher: SetPublisher::new(),
            serializers: AtomicRefCell::new(ForestSerializerTable::new()),
        }
    }
    pub fn add<T>(&self, value: T) -> Arcf<Tree<T>> {
        Arcf::new(self.add_raw(value))
    }
    pub(super) fn add_raw<T>(&self, value: T) -> Tree<T> {
        Tree {
            inner: IcTokenLock::new(self.token.id(), value),
        }
    }
    pub fn get<'a, T: ?Sized>(&'a self, value: &'a Arcf<Tree<T>>) -> &'a T {
        value.inner.read(&self.token)
    }
    pub(super) fn get_raw<'a, T: ?Sized>(&'a self, value: &'a Tree<T>) -> &'a T {
        value.inner.read(&self.token)
    }
    pub fn get_mut<'a, T>(&'a mut self, value: &'a Arcf<Tree<T>>) -> &'a mut T
    where
        T: 'static + ?Sized + Unsize<dyn Sync + Send + Any>,
    {
        self.publisher.send(&ByAddress(value.clone()));
        value.inner.write(&mut self.token)
    }
    pub(super) fn serializers(&self) -> &AtomicRefCell<ForestSerializerTable> {
        &self.serializers
    }
    pub(super) fn subscribe(
        &self,
    ) -> SetSubscriber<HashSet<ByAddress<Arcf<Tree<dyn Sync + Send + Any>>>>> {
        self.publisher.subscribe()
    }
    pub fn id(&self) -> ForestId {
        ForestId {
            token: self.token.id(),
        }
    }
}

impl ForestId {
    pub(super) fn add_raw<T>(&self, value: T) -> Tree<T> {
        Tree {
            inner: IcTokenLock::new(self.token, value),
        }
    }
    pub fn add<T>(&self, value: T) -> Arc<Tree<T>> {
        Arc::new(self.add_raw(value))
    }
}

impl<T: Default> Default for ForestRoot<T> {
    fn default() -> Self {
        ForestRoot::new(Forest::new(), T::default())
    }
}

impl<T: ?Sized> Debug for Tree<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Tree").finish_non_exhaustive()
    }
}
