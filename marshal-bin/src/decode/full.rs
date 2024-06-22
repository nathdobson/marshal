use marshal::context::Context;
use marshal_core::decode::depth_budget::{DepthBudgetDecoder, WithDepthBudget};
use marshal_core::decode::poison::{PoisonDecoder, PoisonWrapper};
use marshal_core::decode::AnyDecoder;
use marshal_core::derive_decoder_for_newtype;

use crate::decode::{BinAnyDecoder, BinDecoderSchema, SimpleBinDecoder};
use crate::DeserializeBin;

pub struct BinDecoder<'de, 's>(PoisonDecoder<DepthBudgetDecoder<SimpleBinDecoder<'de, 's>>>);

derive_decoder_for_newtype!(BinDecoder<'de, 's>(PoisonDecoder<DepthBudgetDecoder<SimpleBinDecoder<'de, 's>>>));

pub struct BinDecoderBuilder<'de, 's> {
    inner: BinDecoder<'de, 's>,
    depth_budget: usize,
}

impl<'de, 's> BinDecoderBuilder<'de, 's> {
    pub fn new(input: &'de [u8], schema: &'s mut BinDecoderSchema) -> Self {
        BinDecoderBuilder {
            inner: BinDecoder(PoisonDecoder::new(DepthBudgetDecoder::new(
                SimpleBinDecoder::new(input, schema),
            ))),
            depth_budget: 100,
        }
    }
    pub fn build<'p>(&'p mut self) -> AnyDecoder<'p, 'de, BinDecoder<'de, 's>> {
        let any = self.inner.0.start(WithDepthBudget::new(
            self.depth_budget,
            BinAnyDecoder::default(),
        ));
        AnyDecoder::new(&mut self.inner, any)
    }
    pub fn deserialize<T: DeserializeBin<'de>>(mut self, ctx: &mut Context) -> anyhow::Result<T> {
        let result = T::deserialize(self.build(), ctx)?;
        self.end()?;
        Ok(result)
    }
    pub fn end(self) -> anyhow::Result<()> {
        Ok(self.inner.0.end()?.end()?.end()?)
    }
}
