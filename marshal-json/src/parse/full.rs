use marshal::context::Context;
use marshal::de::Deserialize;
use marshal_core::decode::depth_budget::{DepthBudgetParser, WithDepthBudget};
use marshal_core::decode::Decoder;
use marshal_core::decode::poison::{PoisonAnyParser, PoisonParser, PoisonState};
use marshal_core::decode::simple::{SimpleAnyParser, SimpleParserAdapter};

use crate::parse::{JsonAnyParser, SimpleJsonParser};

pub type JsonParser<'de> = PoisonParser<DepthBudgetParser<SimpleParserAdapter<SimpleJsonParser<'de>>>>;

pub struct JsonParserBuilder<'de> {
    poison: PoisonState,
    parser: SimpleJsonParser<'de>,
    depth_budget: usize,
}

impl<'de> JsonParserBuilder<'de> {
    pub fn new(input: &'de [u8]) -> Self {
        JsonParserBuilder {
            poison: PoisonState::new(),
            parser: SimpleJsonParser::new(input),
            depth_budget: 100,
        }
    }
    pub fn set_budget(&mut self, depth_budget: usize) -> &mut Self {
        self.depth_budget = depth_budget;
        self
    }
    pub fn build<'p>(&'p mut self) -> <JsonParser<'de> as Decoder<'de>>::AnyDecoder<'p> {
        PoisonAnyParser::new(
            &mut self.poison,
            WithDepthBudget::new(
                self.depth_budget,
                SimpleAnyParser::new(&mut self.parser, JsonAnyParser::default()),
            ),
        )
    }
    pub fn parse<T: Deserialize<'de, JsonParser<'de>>>(
        mut self,
        ctx: &mut Context,
    ) -> anyhow::Result<T> {
        let result = T::deserialize(self.build(), ctx)?;
        self.end()?;
        Ok(result)
    }
    pub fn end(self) -> anyhow::Result<()> {
        self.poison.check()?;
        self.parser.end_parsing()?;
        Ok(())
    }
}

pub fn parse_json<'de, T: Deserialize<'de, JsonParser<'de>>>(
    data: &'de [u8],
    ctx: &mut Context,
) -> anyhow::Result<T> {
    JsonParserBuilder::new(data).parse(ctx)
}
