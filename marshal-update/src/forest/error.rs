use std::fmt::{Debug, Display, Formatter};

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