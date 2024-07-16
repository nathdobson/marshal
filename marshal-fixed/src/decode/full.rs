use marshal::context::Context;
use marshal::decode::{AnySpecDecoder, Decoder};
use marshal::decode::depth_budget::{DepthBudgetDecoder, WithDepthBudget};
use marshal::decode::poison::PoisonDecoder;
use marshal::derive_decoder_for_newtype;
use crate::decode::{FixedAnyDecoder, SimpleFixedSpecDecoder};
use crate::{DeserializeFixed};
pub struct FixedSpecDecoder<'de>(PoisonDecoder<DepthBudgetDecoder<SimpleFixedSpecDecoder<'de>>>);

derive_decoder_for_newtype!(FixedSpecDecoder<'de>(PoisonDecoder<DepthBudgetDecoder<SimpleFixedSpecDecoder<'de>>>));

pub struct FixedDecoderBuilder<'de> {
    inner: FixedSpecDecoder<'de>,
    depth_budget: usize,
}

impl<'de> FixedDecoderBuilder<'de> {
    pub fn new(input: &'de [u8]) -> Self {
        FixedDecoderBuilder {
            inner: FixedSpecDecoder(PoisonDecoder::new(DepthBudgetDecoder::new(
                SimpleFixedSpecDecoder::new(input),
            ))),
            depth_budget: 100,
        }
    }
    pub fn build<'p>(&'p mut self) -> AnySpecDecoder<'p, 'de, FixedSpecDecoder<'de,>> {
        let any = self.inner.0.start(WithDepthBudget::new(
            self.depth_budget,
            FixedAnyDecoder::Any,
        ));
        AnySpecDecoder::new(&mut self.inner, any)
    }
    pub fn deserialize<T: DeserializeFixed>(mut self, ctx: Context) -> anyhow::Result<T> {
        let result = T::deserialize(self.build(), ctx)?;
        self.end()?;
        Ok(result)
    }
    pub fn end(self) -> anyhow::Result<()> {
        Ok(self.inner.0.end()?.end()?.end()?)
    }
}

pub struct FixedDecoder;

impl Decoder for FixedDecoder {
    type SpecDecoder<'de> = FixedSpecDecoder<'de>;
}
