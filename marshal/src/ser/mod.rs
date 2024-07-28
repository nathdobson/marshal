use marshal_core::encode::{AnyEncoder, Encoder};

use crate::context::Context;

pub mod rc;

pub trait Serialize<W: Encoder> {
    fn serialize<'w, 'en>(&self, e: AnyEncoder<'w, 'en, W>, ctx: Context) -> anyhow::Result<()>;
}

fn is_object_safe<W: Encoder, T: Serialize<W>>(x: &T) -> &dyn Serialize<W> {
    x
}
