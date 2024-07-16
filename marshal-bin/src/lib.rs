#![feature(slice_take)]
#![feature(utf16_extra)]
#![deny(unused_must_use)]
#![allow(unused_mut)]
#![allow(dead_code)]
#![feature(test)]
#![feature(cell_update)]
#![feature(never_type)]
#![feature(trait_alias)]
#![feature(coerce_unsized)]
#![feature(unsize)]
#![feature(type_alias_impl_trait)]

use num_derive::FromPrimitive;

use marshal::de::Deserialize;
use marshal::ser::Serialize;

use crate::decode::full::BinDecoder;
use crate::encode::full::BinEncoder;

pub mod decode;
pub mod encode;
#[cfg(test)]
mod test;
mod to_from_vu128;
mod util;

#[derive(Debug, Copy, Clone, Eq, Ord, PartialEq, PartialOrd, Hash, FromPrimitive)]
pub enum TypeTag {
    Unit = 0,
    Bool = 1,
    I8 = 2,
    I16 = 3,
    I32 = 4,
    I64 = 5,
    I128 = 6,
    U8 = 7,
    U16 = 8,
    U32 = 9,
    U64 = 10,
    U128 = 11,
    F32 = 12,
    F64 = 13,
    Char = 14,
    Tuple = 15,
    Struct = 16,
    TupleStruct = 17,
    UnitStruct = 23,
    Enum = 18,
    Seq = 19,
    Map = 20,
    EnumDef = 21,
    String = 22,
    Bytes = 24,
    None = 25,
    Some = 26,
}


pub trait SerializeBin = Serialize<BinEncoder>;
pub trait DeserializeBin = Deserialize<BinDecoder>;
