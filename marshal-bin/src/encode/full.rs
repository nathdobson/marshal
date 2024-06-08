use crate::encode::{BinEncoderSchema, SimpleBinEncoder};
use marshal_core::encode::simple::{SimpleAnyEncoder, SimpleEncoderAdapter};
use marshal_core::encode::Encoder;

pub type BinEncoder<'s> = SimpleEncoderAdapter<SimpleBinEncoder<'s>>;

pub struct BinEncoderBuilder<'s> {
    inner: SimpleBinEncoder<'s>,
}

impl<'s> BinEncoderBuilder<'s> {
    pub fn new(schema: &'s mut BinEncoderSchema) -> Self {
        BinEncoderBuilder {
            inner: SimpleBinEncoder::new(schema),
        }
    }
    pub fn build<'w>(&'w mut self) -> <BinEncoder<'s> as Encoder>::AnyEncoder<'w> {
        SimpleAnyEncoder::new(&mut self.inner, ())
    }
    pub fn end(self) -> anyhow::Result<Vec<u8>> {
        self.inner.end()
    }
}
