use marshal_core::encode::{AnyGenEncoder, GenEncoder};

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

pub trait Serialize<W: GenEncoder> {
    fn serialize<'w, 'en>(&self, e: AnyGenEncoder<'w, 'en, W>, ctx: Context) -> anyhow::Result<()>;
}

fn is_object_safe<W: GenEncoder, T: Serialize<W>>(x: &T) -> &dyn Serialize<W> {
    x
}
