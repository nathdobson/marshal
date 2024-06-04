use std::error::Error;
use std::fmt::{Display, Formatter};

use crate::de::context::DeserializeContext;
use crate::parse::Parser;

mod tuple;
mod json;
pub mod context;

pub trait Deserialize<'de, P: Parser<'de>>: Sized {
    fn deserialize<'p>(
        p: P::AnyParser<'p>,
        context: &DeserializeContext,
    ) -> anyhow::Result<Self>;
}

#[derive(Debug)]
pub struct TypeMismatch;
impl Display for TypeMismatch {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "Actual type did not match expected type")
    }
}
impl Error for TypeMismatch {}
