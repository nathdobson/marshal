#![feature(unsize)]
#![feature(coerce_unsized)]
#![deny(unused_must_use)]

pub use marshal_derive::DeserializeUpdate;
pub use marshal_derive::SerializeStream;
pub use marshal_derive::SerializeUpdate;

mod btree_map;
pub mod de;
pub mod hash_map;
pub mod prim;
pub mod ser;
pub mod tree;
pub mod version;
mod object_map;

