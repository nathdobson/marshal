use marshal::context::Context;
use marshal_core::encode::{AnyEncoder, Encoder};

use crate::encode::{BinEncoderSchema, SimpleBinEncoder};
use crate::SerializeBin;

pub type BinEncoder<'s> = SimpleBinEncoder<'s>;

pub struct BinEncoderBuilder<'s> {
    // poison: PoisonState,
    inner: SimpleBinEncoder<'s>,
}

impl<'s> BinEncoderBuilder<'s> {
    pub fn new(schema: &'s mut BinEncoderSchema) -> Self {
        BinEncoderBuilder {
            // poison: PoisonState::new(),
            inner: SimpleBinEncoder::new(schema),
        }
    }
    pub fn build<'w>(&'w mut self) -> AnyEncoder<'w, BinEncoder<'s>> {
        AnyEncoder::new(&mut self.inner, ())
    }
    pub fn serialize<T: SerializeBin>(
        mut self,
        value: &T,
        ctx: &mut Context,
    ) -> anyhow::Result<Vec<u8>> {
        value.serialize(self.build(), ctx)?;
        self.end()
    }
    pub fn end(self) -> anyhow::Result<Vec<u8>> {
        // self.poison.check()?;
        self.inner.end()
    }
}
