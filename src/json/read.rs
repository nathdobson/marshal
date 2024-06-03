use crate::error::ParseResult;
use crate::json::error::JsonError;
use crate::json::JsonParser;

impl<'de> JsonParser<'de> {
    pub fn try_peek_char(&self) -> ParseResult<Option<u8>> {
        Ok(self.cursor.get(0).cloned())
    }
    pub fn peek_char(&self) -> ParseResult<u8> {
        Ok(self.try_peek_char()?.ok_or(JsonError::Eof)?)
    }
    pub fn try_peek_ahead(&self, n: usize) -> ParseResult<Option<u8>> {
        Ok(self.cursor.get(n).cloned())
    }
    pub fn peek_count(&self, count: usize) -> ParseResult<&'de [u8]> {
        Ok(self.cursor.get(..count).ok_or(JsonError::Eof)?)
    }
    pub fn try_read_char(&mut self) -> ParseResult<Option<u8>> {
        if let Some(a) = self.cursor.take(..1) {
            Ok(Some(a[0]))
        } else {
            Ok(None)
        }
    }
    pub fn read_char(&mut self) -> ParseResult<u8> {
        Ok(self.try_read_char()?.ok_or(JsonError::Eof)?)
    }
    pub fn try_read_match(&mut self, expected: impl FnOnce(u8) -> bool) -> ParseResult<bool> {
        if let Some((a, b)) = self.cursor.split_at_checked(1) {
            if expected(a[0]) {
                self.cursor = b;
                Ok(true)
            } else {
                Ok(false)
            }
        } else {
            Err(JsonError::Eof.into())
        }
    }
    pub fn read_matches(&mut self, expected: impl Fn(u8) -> bool) -> ParseResult<&'de [u8]> {
        let limit = self
            .cursor
            .iter()
            .position(|x| !expected(*x))
            .unwrap_or(self.cursor.len());
        Ok(self.cursor.take(..limit).unwrap())
    }
    pub fn read_count(&mut self, count: usize) -> ParseResult<&'de [u8]> {
        Ok(self.cursor.take(..count).ok_or(JsonError::Eof)?)
    }
    pub fn read_whitespace(&mut self) -> ParseResult<()> {
        self.read_matches(|x| matches!(x, b' ' | b'\n' | b'\r' | b'\t'))?;
        Ok(())
    }
    pub fn try_read_exact(&mut self, expected: u8) -> ParseResult<bool> {
        self.read_whitespace()?;
        if !self.try_read_match(|x| x == expected)? {
            return Ok(false);
        }
        Ok(true)
    }
    pub fn read_exact(&mut self, expected: u8) -> ParseResult<()> {
        self.read_whitespace()?;
        if !self.try_read_match(|x| x == expected)? {
            return Err(JsonError::ExpectedToken {
                expected: char::from(expected),
                found: self.cursor.get(0).map(|x| char::from(*x)),
            }.into());
        }
        Ok(())
    }

    pub fn read_token(&mut self) -> ParseResult<&'de [u8]> {
        Ok(self.read_matches(|x| x.is_ascii_alphabetic())?)
    }

    pub fn read_bool(&mut self) -> ParseResult<bool> {
        match self.read_token()? {
            b"false" => Ok(false),
            b"true" => Ok(true),
            _ => Err(JsonError::UnexpectedIdentifer.into()),
        }
    }

    pub fn read_null(&mut self) -> ParseResult<()> {
        match self.read_token()? {
            b"null" => Ok(()),
            _ => Err(JsonError::UnexpectedIdentifer.into()),
        }
    }

    pub fn end_parsing(&mut self) -> ParseResult<()> {
        self.read_whitespace()?;
        if !self.cursor.is_empty() {
            Err(JsonError::TrailingText.into())
        } else {
            Ok(())
        }
    }
}
