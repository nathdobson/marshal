use marshal::context::Context;
use marshal::de::Deserialize;
use marshal_core::decode::{AnyDecoder};

use crate::decode::{JsonAnyDecoder, SimpleJsonDecoder};

pub type JsonDecoder<'de> = SimpleJsonDecoder<'de>;

pub struct JsonDecoderBuilder<'de> {
    decoder: SimpleJsonDecoder<'de>,
    depth_budget: usize,
}

impl<'de> JsonDecoderBuilder<'de> {
    pub fn new(input: &'de [u8]) -> Self {
        JsonDecoderBuilder {
            decoder: SimpleJsonDecoder::new(input),
            depth_budget: 100,
        }
    }
    pub fn set_budget(&mut self, depth_budget: usize) -> &mut Self {
        self.depth_budget = depth_budget;
        self
    }
    pub fn build<'p>(&'p mut self) -> AnyDecoder<'p, 'de, JsonDecoder<'de>> {
        AnyDecoder::new(&mut self.decoder, JsonAnyDecoder::default())
        // PoisonAnyDecoder::new(
        //     &mut self.poison,
        //     WithDepthBudget::new(
        //         self.depth_budget,
        //         SimpleAnyDecoder::new(&mut self.decoder, JsonAnyDecoder::default()),
        //     ),
        // )
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
        self.decoder.end_parsing()?;
        Ok(())
    }
}
