use marshal::context::Context;
use marshal_core::decode::{AnySpecDecoder, Decoder};
use marshal_core::decode::depth_budget::{DepthBudgetDecoder, WithDepthBudget};
use marshal_core::decode::poison::PoisonDecoder;
use marshal_core::derive_decoder_for_newtype;

use crate::decode::{BinAnyDecoder, BinDecoderSchema, SimpleBinSpecDecoder};
use crate::DeserializeBin;

pub struct BinSpecDecoder<'de>(PoisonDecoder<DepthBudgetDecoder<SimpleBinSpecDecoder<'de>>>);

derive_decoder_for_newtype!(BinSpecDecoder<'de>(PoisonDecoder<DepthBudgetDecoder<SimpleBinSpecDecoder<'de>>>));

pub struct BinDecoderBuilder<'de> {
    inner: BinSpecDecoder<'de>,
    depth_budget: usize,
}

impl<'de> BinDecoderBuilder<'de> {
    pub fn new(input: &'de [u8], schema: &'de mut BinDecoderSchema) -> Self {
        BinDecoderBuilder {
            inner: BinSpecDecoder(PoisonDecoder::new(DepthBudgetDecoder::new(
                SimpleBinSpecDecoder::new(input, schema),
            ))),
            depth_budget: 100,
        }
    }
    pub fn build<'p>(&'p mut self) -> AnySpecDecoder<'p, 'de, BinSpecDecoder<'de,>> {
        let any = self.inner.0.start(WithDepthBudget::new(
            self.depth_budget,
            BinAnyDecoder::default(),
        ));
        AnySpecDecoder::new(&mut self.inner, any)
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

pub struct BinDecoder;

impl Decoder for BinDecoder {
    type SpecDecoder<'de> = BinSpecDecoder<'de>;
}
