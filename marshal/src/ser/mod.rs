use marshal_core::encode::{AnyEncoder, Encoder};

use crate::context::Context;

mod boxed;
mod map;
mod never;
mod number;
mod option;
pub mod rc;
mod reference;
mod string;
mod tuple;
mod vec;
mod result;
mod anyhow_ser;
mod slice;
#[cfg(feature="by_address")]
mod by_address;

pub trait Serialize<W: Encoder> {
    fn serialize<'w, 'en>(&self, e: AnyEncoder<'w, 'en, W>, ctx: Context) -> anyhow::Result<()>;
}

fn is_object_safe<W: Encoder, T: Serialize<W>>(x: &T) -> &dyn Serialize<W> {
    x
}
