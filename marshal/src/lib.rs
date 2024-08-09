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
#![feature(debug_closure_helpers)]
#![feature(trait_upcasting)]

//!
//! An alternative serialization framework for Rust, supporting:
//!  * ✅ Shared references with [Arc](std::sync::Arc) and [Rc](std::rc::Rc).
//!  * ✅ Reference cycles with [sync::Weak](std::sync::Weak) and [rc::Weak](std::rc::Weak).
//!  * ✅ Trait objects with [Box], [Arc](std::sync::Arc), and [Rc](std::rc::Rc).
//!  * ✅ Incremental serialization of changes to data structures.
//!
//! Some features are not supported:
//!  * ❌ Zero-copy deserialization.
//!
//! ## Data Model
//! The data model is equivalent to [Serde's data model](https://serde.rs/data-model.html), except newtypes are just treated as 1-tuples.
//!
//! ## API layers
//! There are three layers at which this API can be used.
//!
//! * Layer 1: [SpecEncoder](encode::SpecEncoder) and [SpecDecoder](decode::SpecDecoder). These are the core traits for specifying a data format.
//!
//! * Layer 2: [AnySpecEncoder](encode::AnySpecEncoder) and [AnySpecDecoder](decode::AnySpecDecoder). These structs provide a wrapper around SpecEncoder and SpecDecoder that are safer and similar to use.
//!
//! * Layer 3 [Serialize](ser::Serialize) and [Deserialize](de::Deserialize). These traits define data structures that can be serialized and deserialized.
//!
//! ### Shared References
//! The [marshal_shared] crate provides one method for handling shared references and reference cycles.
//!
//! ### Trait Objects
//! The [marshal_object] crate provides makes it possible to serialize and deserialize trait objects.
//!
//! ### Incremental Serialization and Deserialization
//! The [marshal_update] crate provides traits to encode incremental updates for a data structure to avoid writing the entire data structure for every change.

extern crate self as marshal;

pub use marshal_core::*;
pub use marshal_derive::Deserialize;
pub use marshal_derive::Serialize;

pub mod context;
pub mod de;
pub mod ser;
mod features;

#[doc(hidden)]
pub mod reexports {
    pub use anyhow;

    pub use marshal_pointer;
}
