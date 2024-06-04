use crate::de::context::DeserializeContext;
use crate::de::Deserialize;
use crate::parse::depth_budget::{DepthBudgetParser, WithDepthBudget};
use crate::parse::json::{JsonParser, SingletonContext};
use crate::parse::poison::{PoisonAnyParser, PoisonParser, PoisonState};
use crate::parse::simple::{SimpleAnyParser, SimpleParserAdapter};
use crate::parse::Parser;

type JsonFullParser<'de> = PoisonParser<DepthBudgetParser<SimpleParserAdapter<JsonParser<'de>>>>;

pub struct JsonFullParserBuilder<'de> {
    poison: PoisonState,
    parser: JsonParser<'de>,
}

impl<'de> JsonFullParserBuilder<'de> {
    pub fn new(input: &'de [u8]) -> Self {
        JsonFullParserBuilder {
            poison: PoisonState::new(),
            parser: JsonParser::new(input),
        }
    }
    pub fn build<'p>(&'p mut self) -> <JsonFullParser<'de> as Parser<'de>>::AnyParser<'p> {
        PoisonAnyParser::new(
            &mut self.poison,
            WithDepthBudget::new(
                100,
                SimpleAnyParser::new(&mut self.parser, SingletonContext::default()),
            ),
        )
    }
    fn end(self) -> anyhow::Result<()> {
        self.poison.check()?;
        self.parser.end_parsing()?;
        Ok(())
    }
}

pub fn parse_json<'de, T: Deserialize<'de, JsonFullParser<'de>>>(
    data: &'de [u8],
    ctx: &DeserializeContext,
) -> anyhow::Result<T> {
    let mut builder = JsonFullParserBuilder::new(data);
    let value = T::deserialize(builder.build(), ctx)?;
    builder.end()?;
    Ok(value)
}
