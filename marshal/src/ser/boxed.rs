use crate::context::Context;
use crate::ser::Serialize;
use marshal_core::encode::Encoder;

impl<E: Encoder, T: Serialize<E>> Serialize<E> for Box<T> {
    fn serialize(&self, w: E::AnyEncoder<'_>, ctx: &mut Context) -> anyhow::Result<()> {
        (**self).serialize(w, ctx)
    }
}
