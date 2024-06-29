use marshal::context::Context;
use marshal_core::derive_encoder_for_newtype;
use marshal_core::encode::poison::PoisonEncoder;
use marshal_core::encode::{AnyEncoder, GenEncoder};

use crate::encode::{BinEncoderSchema, SimpleBinEncoder};
use crate::SerializeBin;

pub struct BinEncoder<'s>(PoisonEncoder<SimpleBinEncoder<'s>>);

derive_encoder_for_newtype!(BinEncoder<'s>(PoisonEncoder<SimpleBinEncoder<'s>>));

pub struct BinEncoderBuilder<'s> {
    inner: BinEncoder<'s>,
}

impl<'s> BinEncoderBuilder<'s> {
    pub fn new(schema: &'s mut BinEncoderSchema) -> Self {
        BinEncoderBuilder {
            inner: BinEncoder(PoisonEncoder::new(SimpleBinEncoder::new(schema))),
        }
    }
    pub fn build<'w>(&'w mut self) -> AnyEncoder<'w, BinEncoder<'s>> {
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

pub struct BinGenEncoder;

impl GenEncoder for BinGenEncoder {
    type Encoder<'en> = BinEncoder<'en>;
}
