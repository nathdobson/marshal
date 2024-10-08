use marshal::context::Context;
use marshal::de::Deserialize;
use marshal_core::decode::{AnySpecDecoder, Decoder};
use marshal_core::decode::depth_budget::{DepthBudgetDecoder, WithDepthBudget};
use marshal_core::decode::poison::PoisonDecoder;
use marshal_core::derive_decoder_for_newtype;

use crate::decode::{JsonAnyDecoder, SimpleJsonSpecDecoder};

pub struct JsonSpecDecoder<'de>(PoisonDecoder<DepthBudgetDecoder<SimpleJsonSpecDecoder<'de>>>);

derive_decoder_for_newtype!(JsonSpecDecoder<'de>(PoisonDecoder<DepthBudgetDecoder<SimpleJsonSpecDecoder<'de>>>));

pub struct JsonDecoderBuilder<'de> {
    decoder: JsonSpecDecoder<'de>,
    depth_budget: usize,
}

impl<'de> JsonDecoderBuilder<'de> {
    pub fn new(input: &'de [u8]) -> Self {
        JsonDecoderBuilder {
            decoder: JsonSpecDecoder(PoisonDecoder::new(DepthBudgetDecoder::new(
                SimpleJsonSpecDecoder::new(input),
            ))),
            depth_budget: 100,
        }
    }
    pub fn set_budget(&mut self, depth_budget: usize) -> &mut Self {
        self.depth_budget = depth_budget;
        self
    }
    pub fn build<'p>(&'p mut self) -> AnySpecDecoder<'p, 'de, JsonSpecDecoder<'de>> {
        let any = JsonAnyDecoder::default();
        let any = WithDepthBudget::new(self.depth_budget, any);
        let any = self.decoder.0.start(any);
        AnySpecDecoder::new(&mut self.decoder, any)
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
    pub fn location(&self) -> String {
        self.decoder.0.inner().inner().location()
    }
    pub fn try_read_eof(&mut self)->anyhow::Result<bool>{
        self.decoder.0.inner_mut().inner_mut().try_read_eof()
    }
    pub fn with<
        F: for<'p> FnOnce(AnySpecDecoder<'p, 'de, JsonSpecDecoder<'de>>) -> anyhow::Result<T>,
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

pub struct JsonDecoder;

impl Decoder for JsonDecoder {
    type SpecDecoder<'de> = JsonSpecDecoder<'de>;
}
