use marshal::context::Context;
use marshal_core::decode::AnyDecoder;
use marshal_core::decode::depth_budget::{DepthBudgetDecoder, WithDepthBudget};
use marshal_core::decode::poison::PoisonDecoder;
use marshal_core::derive_decoder_for_newtype;

use crate::decode::{BinAnyDecoder, BinDecoderSchema, SimpleBinDecoder};
use crate::DeserializeBin;

pub struct BinDecoder<'de>(PoisonDecoder<DepthBudgetDecoder<SimpleBinDecoder<'de>>>);

derive_decoder_for_newtype!(BinDecoder<'de>(PoisonDecoder<DepthBudgetDecoder<SimpleBinDecoder<'de>>>));

pub struct BinDecoderBuilder<'de> {
    inner: BinDecoder<'de>,
    depth_budget: usize,
}

impl<'de> BinDecoderBuilder<'de> {
    pub fn new(input: &'de [u8], schema: &BinDecoderSchema) -> Self {
        BinDecoderBuilder {
            inner: BinDecoder(PoisonDecoder::new(DepthBudgetDecoder::new(
                SimpleBinDecoder::new(input, schema),
            ))),
            depth_budget: 100,
        }
    }
    pub fn build<'p>(&'p mut self) -> AnyDecoder<'p, 'de, BinDecoder<'de>> {
        let any = self.inner.0.start(WithDepthBudget::new(
            self.depth_budget,
            BinAnyDecoder::default(),
        ));
        AnyDecoder::new(&mut self.inner, any)
    }
    pub fn deserialize<T: DeserializeBin<'de>>(mut self, mut ctx: Context) -> anyhow::Result<T> {
        let result = T::deserialize(self.build(), ctx)?;
        self.end()?;
        Ok(result)
    }
    pub fn end(self) -> anyhow::Result<()> {
        Ok(self.inner.0.end()?.end()?.end()?)
    }
}
