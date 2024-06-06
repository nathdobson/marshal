#![feature(slice_take)]
#![feature(utf16_extra)]
#![deny(unused_must_use)]
#![allow(unused_mut)]
#![allow(dead_code)]
#![feature(never_type)]
#![feature(try_blocks)]

pub mod context;
pub mod de;
pub mod ser;

#[doc(hidden)]
pub mod reexports {
    pub use marshal_core;
    pub use anyhow;
}
