#![feature(slice_take)]
#![feature(utf16_extra)]
#![deny(unused_must_use)]
#![allow(unused_mut)]
#![allow(dead_code)]
#![feature(never_type)]
#![feature(try_blocks)]
#![feature(specialization)]
#![feature(adt_const_params)]
#![allow(incomplete_features)]
#![feature(macro_metavar_expr)]

pub mod context;
pub mod de;
pub mod ser;

#[doc(hidden)]
pub mod reexports {
    pub use anyhow;

    pub use marshal_core;
}
