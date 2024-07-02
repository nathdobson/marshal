use std::fmt::{Debug, Display, Formatter};

use marshal_core::decode::{AnyDecoder, Decoder};

use crate::context::Context;

mod boxed;
mod map;
mod never;
mod number;
mod option;
pub mod rc;
mod string;
mod tuple;
mod vec;

pub trait Deserialize<D: Decoder> {
    fn deserialize<'p, 'de>(d: AnyDecoder<'p, 'de, D>, ctx: Context) -> anyhow::Result<Self>
    where
        Self: Sized;
}

fn is_object_safe<D: Decoder, T: Deserialize<D>>(x: &T) -> &dyn Deserialize<D> {
    x
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
