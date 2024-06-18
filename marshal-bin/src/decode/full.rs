use marshal::context::Context;
use marshal_core::decode::{AnyDecoder};

use crate::decode::{BinAnyDecoder, BinDecoderSchema, SimpleBinDecoder};
use crate::DeserializeBin;

pub type BinDecoder<'de, 's> = SimpleBinDecoder<'de, 's>;

pub struct BinDecoderBuilder<'de, 's> {
    inner: SimpleBinDecoder<'de, 's>,
    depth_budget: usize,
}

impl<'de, 's> BinDecoderBuilder<'de, 's> {
    pub fn new(input: &'de [u8], schema: &'s mut BinDecoderSchema) -> Self {
        BinDecoderBuilder {
            inner: SimpleBinDecoder::new(input, schema),
            depth_budget: 100,
        }
    }
    pub fn build<'p>(&'p mut self) -> AnyDecoder<'p, 'de, BinDecoder<'de, 's>> {
        AnyDecoder::new(&mut self.inner, BinAnyDecoder::default())
        // PoisonAnyDecoder::new(
        //     &mut self.poison,
        //     WithDepthBudget::new(
        //         self.depth_budget,
        //         SimpleAnyDecoder::new(&mut self.inner, BinAnyDecoder::Read),
        //     ),
        // )
    }
    pub fn deserialize<T: DeserializeBin<'de>>(mut self, ctx: &mut Context) -> anyhow::Result<T> {
        let result = T::deserialize(self.build(), ctx)?;
        self.end()?;
        Ok(result)
    }
    pub fn end(self) -> anyhow::Result<()> {
        self.inner.end()
    }
}
