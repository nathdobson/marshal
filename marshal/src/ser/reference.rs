use marshal_core::encode::{AnyEncoder, Encoder};

use crate::context::Context;
use crate::ser::Serialize;

impl<'a, E: Encoder, T: Serialize<E>> Serialize<E> for &'a T {
    fn serialize(&self, e: AnyEncoder<'_, E>, ctx: Context) -> anyhow::Result<()> {
        (**self).serialize(e, ctx)
    }
}
