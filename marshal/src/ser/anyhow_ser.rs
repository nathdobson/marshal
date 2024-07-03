use marshal_core::encode::{AnyEncoder, Encoder};

use crate::context::Context;
use crate::ser::Serialize;

impl<E: Encoder> Serialize<E> for anyhow::Error {
    fn serialize<'w, 'en>(&self, e: AnyEncoder<'w, 'en, E>, ctx: Context) -> anyhow::Result<()> {
        <String as Serialize<E>>::serialize(&self.to_string(), e, ctx)
    }
}
