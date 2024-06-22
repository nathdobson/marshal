use marshal::context::Context;
use marshal::de::Deserialize;
use marshal_core::decode::depth_budget::{DepthBudgetDecoder, WithDepthBudget};
use marshal_core::decode::poison::PoisonDecoder;
use marshal_core::decode::AnyDecoder;

use crate::decode::{JsonAnyDecoder, SimpleJsonDecoder};

pub type JsonDecoder<'de> = PoisonDecoder<DepthBudgetDecoder<SimpleJsonDecoder<'de>>>;

pub struct JsonDecoderBuilder<'de> {
    decoder: JsonDecoder<'de>,
    depth_budget: usize,
}

impl<'de> JsonDecoderBuilder<'de> {
    pub fn new(input: &'de [u8]) -> Self {
        JsonDecoderBuilder {
            decoder: PoisonDecoder::new(DepthBudgetDecoder::new(SimpleJsonDecoder::new(input))),
            depth_budget: 100,
        }
    }
    pub fn set_budget(&mut self, depth_budget: usize) -> &mut Self {
        self.depth_budget = depth_budget;
        self
    }
    pub fn build<'p>(&'p mut self) -> AnyDecoder<'p, 'de, JsonDecoder<'de>> {
        let any = JsonAnyDecoder::default();
        let any = WithDepthBudget::new(self.depth_budget, any);
        let any = self.decoder.start(any);
        AnyDecoder::new(&mut self.decoder, any)
    }
    pub fn deserialize<T: Deserialize<'de, JsonDecoder<'de>>>(
        mut self,
        ctx: &mut Context,
    ) -> anyhow::Result<T> {
        let result = T::deserialize(self.build(), ctx)?;
        self.end()?;
        Ok(result)
    }
    pub fn end(self) -> anyhow::Result<()> {
        self.decoder.end()?.end()?.end()?;
        Ok(())
    }
    pub fn with<F: for<'p> FnOnce(AnyDecoder<'p, 'de, JsonDecoder<'de>>) -> anyhow::Result<T>, T>(
        mut self,
        f: F,
    ) -> anyhow::Result<T> {
        let result = f(self.build())?;
        self.end()?;
        Ok(result)
    }
}
