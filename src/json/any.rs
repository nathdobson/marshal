use crate::error::ParseResult;
use crate::json::error::JsonError;
use crate::json::JsonParser;

#[derive(Eq, Ord, PartialEq, PartialOrd, Copy, Clone, Debug, Hash)]
pub enum PeekType {
    String,
    Number,
    Seq,
    Null,
    Map,
    Bool,
}

impl<'de> JsonParser<'de> {
    pub fn peek_type<'p>(&'p mut self) -> ParseResult<PeekType> {
        self.read_whitespace()?;
        let result = match self.peek_char()? {
            b'[' => PeekType::Seq,
            b'"' => PeekType::String,
            b'{' => PeekType::Map,
            b't' | b'f' => PeekType::Bool,
            b'n' => PeekType::Null,
            x if x.is_ascii_digit() || x == b'-' || x == b'.' => PeekType::Number,
            c => {
                return Err(JsonError::UnexpectedInitialCharacter {
                    found: char::from(c),
                }
                .into())
            }
        };
        Ok(result)
    }
}
