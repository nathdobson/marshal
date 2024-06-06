use std::fmt::{Debug, Display, Formatter};

use marshal_core::parse::Parser;

use crate::context::Context;

mod hash_map;
mod string;
mod tuple;
mod vec;
mod number;

pub trait Deserialize<'de, P: Parser<'de>>: Sized {
    fn deserialize<'p>(p: P::AnyParser<'p>, ctx: &mut Context) -> anyhow::Result<Self>;
}

#[derive(Debug)]
pub struct MissingFieldError {
    pub field_name: &'static str,
}

impl Display for MissingFieldError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {

        Debug::fmt(self, f)
    }
}

impl std::error::Error for MissingFieldError {}
