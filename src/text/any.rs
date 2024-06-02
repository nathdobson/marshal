use crate::text::bomb::TextBomb;
use crate::text::depth_budget::DepthBudget;
use crate::text::error::{TextError, TextResult};
use crate::text::map::TextMapParser;
use crate::text::number::TextNumberParser;
use crate::text::seq::TextSeqParser;
use crate::text::TextParser;

#[non_exhaustive]
pub enum TextAny<'p, 'de> {
    String(String),
    Number(TextNumberParser<'p, 'de>),
    TextSeqParser(TextSeqParser<'p, 'de>),
    Null,
    TextMapParser(TextMapParser<'p, 'de>),
    Bool(bool),
}

pub struct TextAnyParser<'p, 'de> {
    parser: TextBomb<'p, 'de>,
    position: TextAnyPosition,
    budget: DepthBudget,
}

pub enum TextAnyPosition {
    Any,
    String,
}

impl<'p, 'de> TextAnyParser<'p, 'de> {
    pub fn new(
        parser: &'p mut TextParser<'de>,
        position: TextAnyPosition,
        budget: DepthBudget,
    ) -> Self {
        TextAnyParser {
            parser: TextBomb::new(parser),
            position,
            budget,
        }
    }
    pub fn parse_any(mut self) -> TextResult<TextAny<'p, 'de>> {
        self.parser.read_whitespace()?;
        let result = match self.parser.peek_char()? {
            b'[' => {
                TextAny::TextSeqParser(TextSeqParser::new(self.parser.into_inner(), self.budget)?)
            }
            b'"' => {
                let s = self.parser.read_string()?;
                self.parser.defuse();
                TextAny::String(s)
            }
            b'{' => {
                TextAny::TextMapParser(TextMapParser::new(self.parser.into_inner(), self.budget)?)
            }
            x if x.is_ascii_alphabetic() => {
                match self.parser.consume_slice(|x| x.is_ascii_alphabetic())? {
                    b"null" => {
                        self.parser.defuse();
                        TextAny::Null
                    }
                    b"false" => {
                        self.parser.defuse();
                        TextAny::Bool(false)
                    }
                    b"true" => {
                        self.parser.defuse();
                        TextAny::Bool(true)
                    }
                    _ => return Err(TextError::UnexpectedIdentifer),
                }
            }
            x if x.is_ascii_digit() || x == b'-' || x == b'.' => {
                TextAny::Number(TextNumberParser::new(self.parser.into_inner()))
            }
            c => {
                return Err(TextError::UnexpectedInitialCharacter {
                    found: char::from(c),
                })
            }
        };
        match &self.position {
            TextAnyPosition::Any => {}
            TextAnyPosition::String => match &result {
                TextAny::String(_) => {}
                _ => return Err(TextError::ExpectedString),
            },
        }
        Ok(result)
    }
}
