use marshal::context::Context;
use marshal::de::Deserialize;
use marshal_core::decode::depth_budget::{DepthBudgetDecoder, WithDepthBudget};
use marshal_core::decode::poison::PoisonDecoder;
use marshal_core::decode::AnyDecoder;
use marshal_core::derive_decoder_for_newtype;
use rc_slice2::ArcSlice;

use crate::decode::{JsonAnyDecoder, SimpleJsonDecoder};

pub struct JsonDecoder(PoisonDecoder<DepthBudgetDecoder<SimpleJsonDecoder>>);

derive_decoder_for_newtype!(JsonDecoder<>(PoisonDecoder<DepthBudgetDecoder<SimpleJsonDecoder>>));

pub struct JsonDecoderBuilder {
    decoder: JsonDecoder,
    depth_budget: usize,
}

impl JsonDecoderBuilder {
    pub fn new(input: ArcSlice<[u8]>) -> Self {
        JsonDecoderBuilder {
            decoder: JsonDecoder(PoisonDecoder::new(DepthBudgetDecoder::new(
                SimpleJsonDecoder::new(input),
            ))),
            depth_budget: 100,
        }
    }
    pub fn set_budget(&mut self, depth_budget: usize) -> &mut Self {
        self.depth_budget = depth_budget;
        self
    }
    pub fn build<'p>(&'p mut self) -> AnyDecoder<'p, JsonDecoder> {
        let any = JsonAnyDecoder::default();
        let any = WithDepthBudget::new(self.depth_budget, any);
        let any = self.decoder.0.start(any);
        AnyDecoder::new(&mut self.decoder, any)
    }
    pub fn deserialize<T: Deserialize<JsonDecoder>>(
        mut self,
        mut ctx: Context,
    ) -> anyhow::Result<T> {
        let result = T::deserialize(self.build(), ctx)?;
        self.end()?;
        Ok(result)
    }
    pub fn end(self) -> anyhow::Result<()> {
        self.decoder.0.end()?.end()?.end()?;
        Ok(())
    }
    pub fn with<F: for<'p> FnOnce(AnyDecoder<'p, JsonDecoder>) -> anyhow::Result<T>, T>(
        mut self,
        f: F,
    ) -> anyhow::Result<T> {
        let result = f(self.build())?;
        self.end()?;
        Ok(result)
    }
}
