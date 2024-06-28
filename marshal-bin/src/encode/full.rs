use marshal::context::Context;
use marshal_core::derive_encoder_for_newtype;
use marshal_core::encode::AnyEncoder;
use marshal_core::encode::poison::PoisonEncoder;

use crate::encode::{BinEncoderSchema, SimpleBinEncoder};
use crate::SerializeBin;

pub struct BinEncoder(PoisonEncoder<SimpleBinEncoder>);

derive_encoder_for_newtype!(BinEncoder(PoisonEncoder<SimpleBinEncoder>));

pub struct BinEncoderBuilder {
    inner: BinEncoder
}

impl BinEncoderBuilder {
    pub fn new(schema: &BinEncoderSchema) -> Self {
        BinEncoderBuilder {
            inner: BinEncoder(PoisonEncoder::new(SimpleBinEncoder::new(schema))),
        }
    }
    pub fn build<'w>(&'w mut self) -> AnyEncoder<'w, BinEncoder> {
        let any = self.inner.0.start(());
        AnyEncoder::new(&mut self.inner, any)
    }
    pub fn serialize<T: SerializeBin>(
        mut self,
        value: &T,
        mut ctx: Context,
    ) -> anyhow::Result<Vec<u8>> {
        value.serialize(self.build(), ctx)?;
        self.end()
    }
    pub fn end(self) -> anyhow::Result<Vec<u8>> {
        self.inner.0.end()?.end()
    }
}
