use crate::text::any::TextAnyPosition;
use crate::text::bomb::TextBomb;
use crate::text::depth_budget::DepthBudget;
use crate::text::error::TextResult;
use crate::text::TextParser;

pub struct TextSeqParser<'p, 'de> {
    parser: TextBomb<'p, 'de>,
    started: bool,
    budget: DepthBudget,
}

impl<'p, 'de> TextSeqParser<'p, 'de> {
    pub fn new(parser: &'p mut TextParser<'de>, budget: DepthBudget) -> TextResult<Self> {
        parser.consume_token(b'[')?;
        Ok(TextSeqParser {
            parser: TextBomb::new(parser),
            started: false,
            budget: budget.child()?,
        })
    }
    pub fn next<'p2>(&'p2 mut self) -> TextResult<Option<TextAnyParser<'p2, 'de>>> {
        if self.parser.try_consume_token(b']')? {
            return Ok(None);
        }
        if self.started {
            self.parser.consume_token(b',')?;
        }
        self.started = true;
        Ok(Some(TextAnyParser::new(
            &mut *self.parser,
            TextAnyPosition::Any,
            self.budget,
        )))
    }
    pub fn end(mut self) -> TextResult<()> {
        self.parser.defuse();
        Ok(())
    }
}
