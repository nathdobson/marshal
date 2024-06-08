#![feature(slice_take)]
#![feature(utf16_extra)]
#![deny(unused_must_use)]
#![allow(unused_mut)]
#![allow(dead_code)]
#![feature(try_blocks)]
#![feature(never_type)]

pub mod parse;
pub mod write;

#[derive(Debug)]
pub enum Primitive {
    Unit,
    Bool(bool),
    I8(i8),
    I16(i16),
    I32(i32),
    I64(i64),
    I128(i128),
    U8(u8),
    U16(u16),
    U32(u32),
    U64(u64),
    U128(u128),
    F32(f32),
    F64(f64),
    Char(char),
}

#[derive(Debug, Copy, Clone, Eq, Ord, PartialEq, PartialOrd, Hash)]
pub enum PrimitiveType {
    Unit,
    Bool,
    I8,
    I16,
    I32,
    I64,
    I128,
    U8,
    U16,
    U32,
    U64,
    U128,
    F32,
    F64,
    Char,
}
impl TryFrom<Primitive> for usize {
    type Error = anyhow::Error;

    fn try_from(value: Primitive) -> Result<Self, Self::Error> {
        Ok(match value {
            Primitive::Unit => 0,
            Primitive::Bool(x) => x as Self,
            Primitive::I8(x) => Self::try_from(x)?,
            Primitive::I16(x) => Self::try_from(x)?,
            Primitive::I32(x) => Self::try_from(x)?,
            Primitive::I64(x) => Self::try_from(x)?,
            Primitive::I128(x) => Self::try_from(x)?,
            Primitive::U8(x) => Self::try_from(x)?,
            Primitive::U16(x) => Self::try_from(x)?,
            Primitive::U32(x) => Self::try_from(x)?,
            Primitive::U64(x) => Self::try_from(x)?,
            Primitive::U128(x) => Self::try_from(x)?,
            Primitive::F32(x) => value.mismatch("u8")?,
            Primitive::F64(x) => value.mismatch("u8")?,
            Primitive::Char(x) => Self::try_from(x as u32)?,
        })
    }
}
