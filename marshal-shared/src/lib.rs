#![feature(trait_upcasting)]
#![feature(coerce_unsized)]

use std::fmt::{Debug, Display, Formatter};

pub mod de;
pub mod ser;

pub mod reexports{
    pub use anyhow;

    pub use marshal;
    pub use marshal_pointer;
}

#[derive(Debug)]
pub enum SharedError {
    UnknownReference,
    DoubleDefinition,
    TypeMismatch,
    MissingDefinition,
}

impl Display for SharedError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        Debug::fmt(self, f)
    }
}

impl std::error::Error for SharedError {}
