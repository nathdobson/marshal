#![feature(trait_alias)]
#![deny(unused_must_use)]

use crate::decode::full::RsonDecoder;
use crate::encode::full::RsonEncoder;
use marshal::de::Deserialize;
use marshal::ser::Serialize;
use std::fmt::{Debug, Display, Formatter};

pub mod decode;
pub mod encode;

pub trait SerializeRson = Serialize<RsonEncoder>;
pub trait DeserializeRson = Deserialize<RsonDecoder>;

#[derive(Debug)]
pub enum RsonError {
    TrailingData,
    UnexpectedEof,
    ExpectedNumber,
    UnexpectedKind { kind: String },
    ExpectedToken { token: &'static str },
    ExpectedIdent,
    UnexpectedEscape,
}

impl Display for RsonError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        Debug::fmt(self, f)
    }
}

impl std::error::Error for RsonError {}
