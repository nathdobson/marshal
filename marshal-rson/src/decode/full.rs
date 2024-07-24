use crate::decode::{RsonAnyDecoder, SimpleRsonSpecDecoder};
use marshal::context::Context;
use marshal::de::Deserialize;
use marshal::decode::depth_budget::{DepthBudgetDecoder, WithDepthBudget};
use marshal::decode::poison::PoisonDecoder;
use marshal::decode::{AnySpecDecoder, Decoder};
use marshal::derive_decoder_for_newtype;
use marshal::reexports::anyhow;

pub struct RsonSpecDecoder<'de>(PoisonDecoder<DepthBudgetDecoder<SimpleRsonSpecDecoder<'de>>>);

derive_decoder_for_newtype!(RsonSpecDecoder<'de>(PoisonDecoder<DepthBudgetDecoder<SimpleRsonSpecDecoder<'de>>>));

pub struct RsonDecoderBuilder<'de> {
    decoder: RsonSpecDecoder<'de>,
    depth_budget: usize,
}

impl<'de> RsonDecoderBuilder<'de> {
    pub fn new(input: &'de str) -> Self {
        RsonDecoderBuilder {
            decoder: RsonSpecDecoder(PoisonDecoder::new(DepthBudgetDecoder::new(
                SimpleRsonSpecDecoder::new(input),
            ))),
            depth_budget: 100,
        }
    }
    pub fn set_budget(&mut self, depth_budget: usize) -> &mut Self {
        self.depth_budget = depth_budget;
        self
    }
    pub fn build<'p>(&'p mut self) -> AnySpecDecoder<'p, 'de, RsonSpecDecoder<'de>> {
        let any = RsonAnyDecoder::new();
        let any = WithDepthBudget::new(self.depth_budget, any);
        let any = self.decoder.0.start(any);
        AnySpecDecoder::new(&mut self.decoder, any)
    }
    pub fn deserialize<T: Deserialize<RsonDecoder>>(mut self, ctx: Context) -> anyhow::Result<T> {
        let result = T::deserialize(self.build(), ctx)?;
        self.end()?;
        Ok(result)
    }
    pub fn end(self) -> anyhow::Result<()> {
        self.decoder.0.end()?.end()?.end()?;
        Ok(())
    }
    pub fn location(&self) -> String {
        self.decoder.0.inner().inner().location()
    }
    pub fn try_read_eof(&mut self) -> anyhow::Result<bool> {
        self.decoder.0.inner_mut().inner_mut().try_read_eof()
    }
    pub fn with<
        F: for<'p> FnOnce(AnySpecDecoder<'p, 'de, RsonSpecDecoder<'de>>) -> anyhow::Result<T>,
        T,
    >(
        mut self,
        f: F,
    ) -> anyhow::Result<T> {
        let result =
            f(self.build()).map_err(|e| e.context(self.decoder.0.inner().inner().location()))?;
        self.end()?;
        Ok(result)
    }
}

pub struct RsonDecoder;

impl Decoder for RsonDecoder {
    type SpecDecoder<'de> = RsonSpecDecoder<'de>;
}
