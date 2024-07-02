#![deny(unused_must_use)]

use anyhow::{anyhow, Error};
use std::fmt::{Display, Formatter};

mod de;
mod ser;

pub struct SerdeWrapper<T> {
    inner: T,
}

impl<T> SerdeWrapper<T> {
    pub fn new(inner: T) -> Self {
        SerdeWrapper { inner }
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
