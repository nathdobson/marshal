use std::fmt::{Debug, Display, Formatter};

pub mod de;
pub mod ser;

pub mod reexports{
    pub use marshal;
    pub use anyhow;
}

#[derive(Debug)]
pub enum SharedError {
    UnknownReference(usize),
    TypeMismatch,
}

impl Display for SharedError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        Debug::fmt(self, f)
    }
}

impl std::error::Error for SharedError {}
