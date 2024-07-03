#![deny(unused_must_use)]

use std::fmt::{Display, Formatter};
use std::ops::{Deref, DerefMut};

use anyhow::{anyhow, Error};

mod de;
mod ser;

#[derive(Clone, Copy, Eq, Ord, PartialEq, PartialOrd, Hash, Debug)]
pub struct WithSerde<T: ?Sized> {
    inner: T,
}

impl<T> WithSerde<T> {
    pub fn new(inner: T) -> Self {
        WithSerde { inner }
    }
    pub fn into_inner(self) -> T {
        self.inner
    }
}

impl<T: ?Sized> Deref for WithSerde<T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<T: ?Sized> DerefMut for WithSerde<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

#[derive(Debug)]
struct MarshalError(anyhow::Error);

impl std::error::Error for MarshalError {}

impl Display for MarshalError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl serde::ser::Error for MarshalError {
    fn custom<T>(msg: T) -> Self
    where
        T: Display,
    {
        MarshalError(anyhow!("{}", msg))
    }
}

impl serde::de::Error for MarshalError {
    fn custom<T>(msg: T) -> Self
    where
        T: Display,
    {
        MarshalError(anyhow!("{}", msg))
    }
}

impl From<anyhow::Error> for MarshalError {
    fn from(value: Error) -> Self {
        MarshalError(value)
    }
}
