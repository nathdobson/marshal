mod tuple;
mod json;

use crate::context::DeserializeContext;
use crate::error::ParseError;
use std::error::Error;
use std::fmt::{Display, Formatter};
use crate::Parser;

pub trait Deserialize<'de, P: Parser<'de>>: Sized {
    fn deserialize<'p>(
        p: P::AnyParser<'p>,
        context: &DeserializeContext,
    ) -> Result<Self, ParseError>;
}

#[derive(Debug)]
pub struct TypeMismatch;
impl Display for TypeMismatch {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "Actual type did not match expected type")
    }
}
impl Error for TypeMismatch {}
