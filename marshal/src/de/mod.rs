use std::error::Error;
use std::fmt::{Display, Formatter};
use marshal_core::parse::Parser;
use crate::context::Context;

mod tuple;
mod vec;
mod hash_map;
mod string;

pub trait Deserialize<'de, P: Parser<'de>>: Sized {
    fn deserialize<'p>(p: P::AnyParser<'p>, ctx: &mut Context) -> anyhow::Result<Self>;
}

