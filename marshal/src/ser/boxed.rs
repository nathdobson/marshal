use marshal_core::encode::Encoder;
use marshal_pointer::boxed::BoxRef;

use crate::context::Context;
use crate::ser::Serialize;

impl<E: Encoder, T: Serialize<E>> Serialize<E> for Box<T> {
    fn serialize(&self, w: E::AnyEncoder<'_>, ctx: &mut Context) -> anyhow::Result<()> {
        (**self).serialize(w, ctx)
    }
}

impl<E: Encoder, T: Serialize<E>> Serialize<E> for BoxRef<T> {
    fn serialize(&self, w: E::AnyEncoder<'_>, ctx: &mut Context) -> anyhow::Result<()> {
        (**self).serialize(w, ctx)
    }
}
