use marshal::context::Context;
use marshal_core::derive_encoder_for_newtype;
use marshal_core::encode::{AnySpecEncoder, Encoder};
use marshal_core::encode::poison::PoisonEncoder;

use crate::encode::{BinEncoderSchema, SimpleBinSpecEncoder};
use crate::SerializeBin;

pub struct BinSpecEncoder<'s>(PoisonEncoder<SimpleBinSpecEncoder<'s>>);

derive_encoder_for_newtype!(BinSpecEncoder<'s>(PoisonEncoder<SimpleBinSpecEncoder<'s>>));

pub struct BinEncoderBuilder<'s> {
    inner: BinSpecEncoder<'s>,
}

impl<'s> BinEncoderBuilder<'s> {
    pub fn new(schema: &'s mut BinEncoderSchema) -> Self {
        BinEncoderBuilder {
            inner: BinSpecEncoder(PoisonEncoder::new(SimpleBinSpecEncoder::new(schema))),
        }
    }
    pub fn build<'w>(&'w mut self) -> AnySpecEncoder<'w, BinSpecEncoder<'s>> {
        let any = self.inner.0.start(());
        AnySpecEncoder::new(&mut self.inner, any)
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

pub struct BinEncoder;

impl Encoder for BinEncoder {
    type SpecEncoder<'en> = BinSpecEncoder<'en>;
}
