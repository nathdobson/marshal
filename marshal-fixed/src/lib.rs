#![feature(never_type)]
#![feature(trait_alias)]
#![feature(slice_take)]

use std::fmt::{Display, Formatter};
use marshal::de::Deserialize;
use marshal::ser::Serialize;
use crate::decode::full::FixedDecoder;
use crate::encode::full::FixedEncoder;

pub mod decode;
pub mod encode;

#[derive(Debug)]
pub enum FixedError {
    UnsupportedHint,
    TrailingData,
    NonZeroPadding,
    UnexpectedEof,
}

impl Display for FixedError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl std::error::Error for FixedError {}

enum DiscriminantWidth {
    U8,
    U16,
    U32,
    U64,
}

impl DiscriminantWidth {
    #[inline]
    fn from_max(max: usize) -> Self {
        if max <= 0x1_00 {
            Self::U8
        } else if max <= 0x1_00_00 {
            Self::U16
        } else if max as u64 <= 0x1_00_00_00_00 {
            Self::U32
        } else {
            Self::U64
        }
    }
}


pub trait SerializeFixed = Serialize<FixedEncoder>;
pub trait DeserializeFixed = Deserialize<FixedDecoder>;
