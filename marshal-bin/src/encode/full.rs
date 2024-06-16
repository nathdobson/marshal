use marshal::context::Context;
use marshal_core::encode::Encoder;
use marshal_core::encode::poison::{PoisonAnyEncoder, PoisonEncoder, PoisonState};
use marshal_core::encode::simple::{SimpleAnyEncoder, SimpleEncoderAdapter};

use crate::encode::{BinEncoderSchema, SimpleBinEncoder};
use crate::SerializeBin;

pub type BinEncoder<'s> = PoisonEncoder<SimpleEncoderAdapter<SimpleBinEncoder<'s>>>;

pub struct BinEncoderBuilder<'s> {
    poison: PoisonState,
    inner: SimpleBinEncoder<'s>,
}

impl<'s> BinEncoderBuilder<'s> {
    pub fn new(schema: &'s mut BinEncoderSchema) -> Self {
        BinEncoderBuilder {
            poison: PoisonState::new(),
            inner: SimpleBinEncoder::new(schema),
        }
    }
    pub fn build<'w>(&'w mut self) -> <BinEncoder<'s> as Encoder>::AnyEncoder<'w> {
        PoisonAnyEncoder::new(&mut self.poison, SimpleAnyEncoder::new(&mut self.inner, ()))
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
        self.poison.check()?;
        self.inner.end()
    }
}
