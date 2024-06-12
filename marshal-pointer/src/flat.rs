use std::ops::Deref;

use crate::AsFlatRef;

pub struct Flat<T>(T);

impl<T> Flat<T> {
    pub fn new(x: T) -> Self {
        Flat(x)
    }
    pub fn into_inner(self) -> T {
        self.0
    }
}

impl<T: AsFlatRef> Deref for Flat<T> {
    type Target = T::FlatRef;
    fn deref(&self) -> &Self::Target {
        self.0.as_flat_ref()
    }
}
