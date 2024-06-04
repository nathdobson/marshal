use serde_json::{Number, Value};

use crate::context::DeserializeContext;
use crate::depth_budget::{DepthBudgetParser, WithDepthBudget};
use crate::deserialize::Deserialize;
use crate::error::{ParseError, ParseResult};
use crate::json::{JsonParser, SingletonContext};
use crate::poison::{PoisonAnyParser, PoisonParser, PoisonState};
use crate::simple::{SimpleAnyParser, SimpleParserAdapter};
use crate::{AnyParser, EntryParser, MapParser, ParseHint, Parser, ParserView, SeqParser};

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
    fn end(self) -> ParseResult<()> {
        self.poison.check()?;
        self.parser.end_parsing()?;
        Ok(())
    }
}

pub fn parse_json<'de, T: Deserialize<'de, JsonFullParser<'de>>>(
    data: &'de [u8],
    ctx: &DeserializeContext,
) -> ParseResult<T> {
    let mut builder = JsonFullParserBuilder::new(data);
    let value = T::deserialize(builder.build(), ctx)?;
    builder.end()?;
    Ok(value)
}
