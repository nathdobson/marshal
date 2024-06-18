use marshal::context::Context;
use marshal::ser::Serialize;
use marshal_core::encode::poison::PoisonEncoder;
use marshal_core::encode::AnyEncoder;

use crate::encode::{BinEncoderSchema, SimpleBinEncoder};
use crate::SerializeBin;

pub type BinEncoder<'s> = PoisonEncoder<SimpleBinEncoder<'s>>;

pub struct BinEncoderBuilder<'s> {
    inner: PoisonEncoder<SimpleBinEncoder<'s>>,
}

impl<'s> BinEncoderBuilder<'s> {
    pub fn new(schema: &'s mut BinEncoderSchema) -> Self {
        BinEncoderBuilder {
            inner: PoisonEncoder::new(SimpleBinEncoder::new(schema)),
        }
    }
    pub fn build<'w>(&'w mut self) -> AnyEncoder<'w, BinEncoder<'s>> {
        let any = self.inner.start(());
        AnyEncoder::new(&mut self.inner, any)
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
        self.inner.end()?.end()
    }
}
