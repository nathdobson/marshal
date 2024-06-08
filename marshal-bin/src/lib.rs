#![feature(slice_take)]
#![feature(utf16_extra)]
#![deny(unused_must_use)]
#![allow(unused_mut)]
#![allow(dead_code)]
#![feature(test)]
#![feature(cell_update)]

use num_derive::FromPrimitive;

mod read;
#[cfg(test)]
mod test;
mod to_from_vu128;
mod util;
mod write;

pub const VU128_MAX_PADDING: usize = 17;

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
}
