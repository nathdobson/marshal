use marshal::context::Context;
use marshal::de::Deserialize;
use marshal_core::decode::depth_budget::{DepthBudgetDecoder, WithDepthBudget};
use marshal_core::decode::Decoder;
use marshal_core::decode::poison::{PoisonAnyDecoder, PoisonDecoder, PoisonState};
use marshal_core::decode::simple::{SimpleAnyDecoder, SimpleDecoderAdapter};

use crate::decode::{JsonAnyDecoder, SimpleJsonDecoder};

pub type JsonDecoder<'de> = PoisonDecoder<DepthBudgetDecoder<SimpleDecoderAdapter<SimpleJsonDecoder<'de>>>>;

pub struct JsonDecoderBuilder<'de> {
    poison: PoisonState,
    decoder: SimpleJsonDecoder<'de>,
    depth_budget: usize,
}

impl<'de> JsonDecoderBuilder<'de> {
    pub fn new(input: &'de [u8]) -> Self {
        JsonDecoderBuilder {
            poison: PoisonState::new(),
            decoder: SimpleJsonDecoder::new(input),
            depth_budget: 100,
        }
    }
    pub fn set_budget(&mut self, depth_budget: usize) -> &mut Self {
        self.depth_budget = depth_budget;
        self
    }
    pub fn build<'p>(&'p mut self) -> <JsonDecoder<'de> as Decoder<'de>>::AnyDecoder<'p> {
        PoisonAnyDecoder::new(
            &mut self.poison,
            WithDepthBudget::new(
                self.depth_budget,
                SimpleAnyDecoder::new(&mut self.decoder, JsonAnyDecoder::default()),
            ),
        )
    }
    pub fn decode<T: Deserialize<'de, JsonDecoder<'de>>>(
        mut self,
        ctx: &mut Context,
    ) -> anyhow::Result<T> {
        let result = T::deserialize(self.build(), ctx)?;
        self.end()?;
        Ok(result)
    }
    pub fn end(self) -> anyhow::Result<()> {
        self.poison.check()?;
        self.decoder.end_parsing()?;
        Ok(())
    }
}

pub fn decode_json<'de, T: Deserialize<'de, JsonDecoder<'de>>>(
    data: &'de [u8],
    ctx: &mut Context,
) -> anyhow::Result<T> {
    JsonDecoderBuilder::new(data).decode(ctx)
}
