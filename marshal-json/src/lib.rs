#![feature(slice_take)]
#![feature(utf16_extra)]
#![deny(unused_must_use)]
#![allow(unused_mut)]
#![allow(dead_code)]
#![feature(never_type)]
#![feature(trait_alias)]
#![feature(coerce_unsized)]
#![feature(unsize)]

use marshal::de::Deserialize;
use marshal::ser::Serialize;

use crate::decode::full::JsonDecoder;
use crate::encode::full::JsonEncoder;

pub mod decode;
pub mod encode;
#[cfg(test)]
mod test;
pub mod value;
pub mod json_object;

#[doc(hidden)]
pub mod reexports{
    pub use anyhow;

    pub use marshal;
    pub use marshal_object;
    pub use marshal_pointer;
}

pub trait SerializeJson = Serialize<JsonEncoder>;
pub trait DeserializeJson<'de> = Deserialize<'de, JsonDecoder<'de>>;
