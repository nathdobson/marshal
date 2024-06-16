use marshal_core::encode::Encoder;

use crate::context::Context;
use crate::ser::Serialize;

impl<'a, E: Encoder, T: Serialize<E>> Serialize<E> for &'a T {
    fn serialize(&self, e: E::AnyEncoder<'_>, ctx: &mut Context) -> anyhow::Result<()> {
        (**self).serialize(e, ctx)
    }
}
