use atomic_refcell::{AtomicRefCell, AtomicRefMut};
use std::collections::hash_set::Drain;
use std::collections::{BTreeSet, HashSet};
use std::hash::Hash;
use std::ops::{Deref, DerefMut};
use std::sync::{Arc, Weak};
use weak_table::PtrWeakHashSet;
pub struct SetSubscriber<S>(Arc<AtomicRefCell<S>>);

pub struct SetPublisher<S> {
    queues: AtomicRefCell<PtrWeakHashSet<Weak<AtomicRefCell<S>>>>,
}

impl<S> SetPublisher<S> {
    pub fn new() -> Self {
        SetPublisher {
            queues: AtomicRefCell::new(PtrWeakHashSet::new()),
        }
    }
    pub fn subscribe(&self) -> SetSubscriber<S>
    where
        S: Default,
    {
        let arc = Arc::new(AtomicRefCell::new(S::default()));
        self.queues.borrow_mut().insert(arc.clone());
        SetSubscriber(arc)
    }
    pub fn send_with<F: FnMut(&mut S)>(&mut self, mut f: F) {
        for queue in self.queues.get_mut().iter() {
            f(&mut *queue.borrow_mut())
        }
    }
}

impl<K: Eq + Hash + Clone> SetPublisher<HashSet<K>> {
    pub fn send(&mut self, k: &K) {
        self.send_with(|map| {
            map.insert(k.clone());
        })
    }
}

impl<K: Ord + Clone> SetPublisher<BTreeSet<K>> {
    pub fn send(&mut self, k: &K) {
        self.send_with(|map| {
            map.insert(k.clone());
        })
    }
}

impl<S> SetSubscriber<S> {
    pub fn recv<'a>(&'a mut self) -> impl 'a + DerefMut<Target = S> {
        self.0.borrow_mut()
    }
}
