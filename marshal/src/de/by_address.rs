use std::ops::Deref;
use crate::context::Context;
use crate::ser::Serialize;
use by_address::ByAddress;
use marshal_core::encode::{AnyEncoder, Encoder};

impl<E: Encoder, T: Deref + Serialize<E>> Serialize<E> for ByAddress<T> {
    fn serialize<'w, 'en>(&self, e: AnyEncoder<'w, 'en, E>, ctx: Context) -> anyhow::Result<()> {
        self.0.serialize(e, ctx)
    }
}
