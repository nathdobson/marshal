use crate::text::any::{TextAnyParser, TextAnyPosition};
use crate::text::bomb::TextBomb;
use crate::text::depth_budget::DepthBudget;
use crate::text::error::{TextError, TextResult};
use crate::text::TextParser;

pub struct TextMapParser<'p, 'de> {
    parser: TextBomb<'p, 'de>,
    started: bool,
    budget: DepthBudget,
}

pub struct TextEntryParser<'p, 'de> {
    parser: TextBomb<'p, 'de>,
    read_key: bool,
    read_value: bool,
    budget: DepthBudget,
}

impl<'p, 'de> TextMapParser<'p, 'de> {
    pub fn new(parser: &'p mut TextParser<'de>, budget: DepthBudget) -> TextResult<Self> {
        parser.consume_token(b'{')?;
        Ok(TextMapParser {
            parser: TextBomb::new(parser),
            started: false,
            budget: budget.child()?,
        })
    }
    pub fn next<'p2>(&'p2 mut self) -> TextResult<Option<TextEntryParser<'p2, 'de>>> {
        if self.parser.try_consume_token(b'}')? {
            return Ok(None);
        }
        if self.started {
            self.parser.consume_token(b',')?;
        }
        self.started = true;
        Ok(Some(TextEntryParser::new(&mut *self.parser, self.budget)))
    }
    pub fn end(mut self) -> TextResult<()> {
        self.parser.defuse();
        Ok(())
    }
}

impl<'p, 'de> TextEntryParser<'p, 'de> {
    pub fn new(parser: &'p mut TextParser<'de>, budget: DepthBudget) -> Self {
        TextEntryParser {
            parser: TextBomb::new(parser),
            read_key: false,
            read_value: false,
            budget,
        }
    }
    pub fn parse_key<'p2>(&'p2 mut self) -> TextResult<TextAnyParser<'p2, 'de>> {
        if self.read_key {
            return Err(TextError::BadState);
        }
        self.read_key = true;
        Ok(TextAnyParser::new(
            &mut *self.parser,
            TextAnyPosition::String,
            self.budget,
        ))
    }
    pub fn parse_value(mut self) -> TextResult<TextAnyParser<'p, 'de>> {
        if !self.read_key || self.read_value {
            return Err(TextError::BadState);
        }
        self.read_value = true;
        self.parser.consume_token(b':')?;
        Ok(TextAnyParser::new(
            self.parser.into_inner(),
            TextAnyPosition::Any,
            self.budget,
        ))
    }
}
