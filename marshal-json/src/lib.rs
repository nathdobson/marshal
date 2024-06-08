#![feature(slice_take)]
#![feature(utf16_extra)]
#![deny(unused_must_use)]
#![allow(unused_mut)]
#![allow(dead_code)]
#![feature(never_type)]

pub mod decode;
pub mod encode;
pub mod value;
#[cfg(test)]
mod test;
