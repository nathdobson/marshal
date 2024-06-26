use marshal_core::encode::{AnyEncoder,  Encoder};
use marshal_pointer::boxed::BoxRef;

use crate::context::Context;
use crate::ser::Serialize;

impl<E: Encoder, T: Serialize<E>> Serialize<E> for Box<T> {
    fn serialize<'w, 'en>(&self, w: AnyEncoder<'w, 'en, E>, ctx: Context) -> anyhow::Result<()> {
        (**self).serialize(w, ctx)
    }
}

impl<E: Encoder, T: Serialize<E>> Serialize<E> for BoxRef<T> {
    fn serialize<'w, 'en>(&self, w: AnyEncoder<'w, 'en, E>, ctx: Context) -> anyhow::Result<()> {
        (**self).serialize(w, ctx)
    }
}
