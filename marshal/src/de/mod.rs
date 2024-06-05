use std::error::Error;
use std::fmt::{Display, Formatter};
use crate::context::Context;

use crate::parse::Parser;

mod tuple;
mod vec;
mod hash_map;
mod string;

pub trait Deserialize<'de, P: Parser<'de>>: Sized {
    fn deserialize<'p>(p: P::AnyParser<'p>, ctx: &mut Context) -> anyhow::Result<Self>;
}

#[derive(Debug)]
pub struct TypeMismatch {
    pub found: &'static str,
    pub expected: &'static str,
}

impl Display for TypeMismatch {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "Actual type did not match expected type")
    }
}
impl Error for TypeMismatch {}
