use std::fmt::{Debug, Display, Formatter};

use marshal_core::decode::Decoder;

use crate::context::Context;

mod map;
mod number;
mod string;
mod tuple;
mod vec;
mod never;
mod option;
mod boxed;
pub mod rc;

pub trait Deserialize<'de, P: Decoder<'de>>: Sized {
    fn deserialize<'p>(p: P::AnyDecoder<'p>, ctx: &mut Context) -> anyhow::Result<Self>;
}

#[derive(Debug)]
pub enum SchemaError {
    MissingField { field_name: &'static str },
    UnknownVariant,
    TupleTooShort,
    UninhabitedType,
    TupleTooLong,
}

impl Display for SchemaError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        Debug::fmt(self, f)
    }
}

impl std::error::Error for SchemaError {}
