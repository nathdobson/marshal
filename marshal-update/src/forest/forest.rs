use crate::forest::de::ForestDeserializerTable;
use crate::forest::ser::ForestSerializerTable;
use crate::ser::set_channel::{SetPublisher, SetSubscriber};
use atomic_refcell::AtomicRefCell;
use by_address::ByAddress;
use std::any::Any;
use std::collections::HashSet;
use std::marker::Unsize;
use std::sync::Arc;
use tokenlock::{IcToken, IcTokenId, IcTokenLock};

#[derive(Copy, Clone)]
pub struct ForestId {
    token: IcTokenId,
}

impl ForestId {
    pub fn add<T>(&self, value: T) -> Tree<T> {
        Tree {
            inner: IcTokenLock::new(self.token, value),
        }
    }
}

pub struct Forest {
    token: IcToken,
    publisher: SetPublisher<HashSet<ByAddress<Arc<Tree<dyn Sync + Send + Any>>>>>,
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
    pub fn new_raw(forest: Forest, deserializers: ForestDeserializerTable, root: T) -> Self {
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
}

impl Forest {
    pub fn new() -> Self {
        Forest {
            token: IcToken::new(),
            publisher: SetPublisher::new(),
            serializers: AtomicRefCell::new(ForestSerializerTable::new()),
        }
    }
    pub fn add<T>(&self, value: T) -> Arc<Tree<T>> {
        Arc::new(self.add_raw(value))
    }
    pub fn add_raw<T>(&self, value: T) -> Tree<T> {
        Tree {
            inner: IcTokenLock::new(self.token.id(), value),
        }
    }
    pub fn get<'a, T: ?Sized>(&'a self, value: &'a Arc<Tree<T>>) -> &'a T {
        value.inner.read(&self.token)
    }
    pub fn get_raw<'a, T: ?Sized>(&'a self, value: &'a Tree<T>) -> &'a T {
        value.inner.read(&self.token)
    }
    pub fn get_mut<'a, T: 'static + ?Sized + Sync + Send>(
        &'a mut self,
        value: &'a Arc<Tree<T>>,
    ) -> &'a mut T
    where
        T: Unsize<dyn Sync + Send + Any>,
    {
        self.publisher.send(&ByAddress(value.clone()));
        value.inner.write(&mut self.token)
    }
    pub fn serializers(&self) -> &AtomicRefCell<ForestSerializerTable> {
        &self.serializers
    }
    pub fn subscribe(&self) -> SetSubscriber<HashSet<ByAddress<Arc<Tree<dyn Sync + Send + Any>>>>> {
        self.publisher.subscribe()
    }
    pub fn id(&self) -> ForestId {
        ForestId {
            token: self.token.id(),
        }
    }
}
