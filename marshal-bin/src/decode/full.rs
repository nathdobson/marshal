use marshal::context::Context;
use marshal_core::decode::depth_budget::{DepthBudgetDecoder, WithDepthBudget};
use marshal_core::decode::poison::PoisonDecoder;
use marshal_core::decode::AnyDecoder;
use marshal_core::derive_decoder_for_newtype;
use rc_slice2::ArcSlice;

use crate::decode::{BinAnyDecoder, BinDecoderSchema, SimpleBinDecoder};
use crate::DeserializeBin;

pub struct BinDecoder(PoisonDecoder<DepthBudgetDecoder<SimpleBinDecoder>>);

derive_decoder_for_newtype!(BinDecoder<>(PoisonDecoder<DepthBudgetDecoder<SimpleBinDecoder>>));

pub struct BinDecoderBuilder {
    inner: BinDecoder,
    depth_budget: usize,
}

impl BinDecoderBuilder {
    pub fn new(input: ArcSlice<[u8]>, schema: &BinDecoderSchema) -> Self {
        BinDecoderBuilder {
            inner: BinDecoder(PoisonDecoder::new(DepthBudgetDecoder::new(
                SimpleBinDecoder::new(input, schema),
            ))),
            depth_budget: 100,
        }
    }
    pub fn build<'p>(&'p mut self) -> AnyDecoder<'p, BinDecoder> {
        let any = self.inner.0.start(WithDepthBudget::new(
            self.depth_budget,
            BinAnyDecoder::default(),
        ));
        AnyDecoder::new(&mut self.inner, any)
    }
    pub fn deserialize<T: DeserializeBin>(mut self, mut ctx: Context) -> anyhow::Result<T> {
        let result = T::deserialize(self.build(), ctx)?;
        self.end()?;
        Ok(result)
    }
    pub fn end(self) -> anyhow::Result<()> {
        Ok(self.inner.0.end()?.end()?.end()?)
    }
}
