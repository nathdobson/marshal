#![feature(slice_take)]
#![feature(utf16_extra)]
#![deny(unused_must_use)]
#![allow(unused_mut)]
#![allow(dead_code)]
#![feature(never_type)]

mod decode;
mod encode;
mod value;
#[cfg(test)]
mod test;
