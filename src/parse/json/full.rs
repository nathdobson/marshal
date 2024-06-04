use crate::de::context::DeserializeContext;
use crate::de::Deserialize;
use crate::parse::depth_budget::{DepthBudgetParser, WithDepthBudget};
use crate::parse::json::{AnyParser, SimpleJsonParser};
use crate::parse::poison::{PoisonAnyParser, PoisonParser, PoisonState};
use crate::parse::simple::{SimpleAnyParser, SimpleParserAdapter};
use crate::parse::Parser;

type JsonParser<'de> = PoisonParser<DepthBudgetParser<SimpleParserAdapter<SimpleJsonParser<'de>>>>;

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
    pub fn build<'p>(&'p mut self) -> <JsonParser<'de> as Parser<'de>>::AnyParser<'p> {
        PoisonAnyParser::new(
            &mut self.poison,
            WithDepthBudget::new(
                self.depth_budget,
                SimpleAnyParser::new(&mut self.parser, AnyParser::default()),
            ),
        )
    }
    pub fn parse<T: Deserialize<'de, JsonParser<'de>>>(
        mut self,
        ctx: &DeserializeContext,
    ) -> anyhow::Result<T> {
        let result = T::deserialize(self.build(), ctx)?;
        self.end()?;
        Ok(result)
    }
    fn end(self) -> anyhow::Result<()> {
        self.poison.check()?;
        self.parser.end_parsing()?;
        Ok(())
    }
}

pub fn parse_json<'de, T: Deserialize<'de, JsonParser<'de>>>(
    data: &'de [u8],
    ctx: &DeserializeContext,
) -> anyhow::Result<T> {
    JsonParserBuilder::new(data).parse(ctx)
}
