use std::str::FromStr;
use crate::error::{ParseError, ParseResult};

use crate::json::error::JsonError;
use crate::json::JsonParser;

struct SliceParser<'p, 'de> {
    parser: &'p mut JsonParser<'de>,
    index: usize,
}

impl<'p, 'de> SliceParser<'p, 'de> {
    pub fn new(parser: &'p mut JsonParser<'de>) -> Self {
        SliceParser { parser, index: 0 }
    }
    pub fn try_consume_char(
        &mut self,
        expected: impl FnOnce(u8) -> bool,
    ) -> ParseResult<Option<u8>> {
        Ok(match self.parser.try_peek_ahead(self.index)? {
            None => None,
            Some(c) => {
                if expected(c) {
                    self.index += 1;
                    Some(c)
                } else {
                    None
                }
            }
        })
    }
    pub fn try_consume_digit(&mut self) -> ParseResult<Option<u8>> {
        Ok(self.try_consume_char(|x| x.is_ascii_digit())?)
    }

    pub fn end(self) -> ParseResult<&'de [u8]> {
        self.parser.read_count(self.index)
    }
}

impl<'de> JsonParser<'de> {
    pub fn read_number<T: FromStr>(&mut self) -> ParseResult<T>
    where
        ParseError: From<T::Err>,
    {
        let mut slice = SliceParser::new(self);
        slice.try_consume_char(|x| x == b'-')?;
        match slice
            .try_consume_digit()?
            .ok_or(JsonError::UnexpectedInput)?
        {
            b'0' => {}
            _ => while slice.try_consume_digit()?.is_some() {},
        }
        if slice.try_consume_char(|x| x == b'.')?.is_some() {
            slice
                .try_consume_digit()?
                .ok_or(JsonError::UnexpectedInput)?;
            while slice.try_consume_digit()?.is_some() {}
        }
        if slice
            .try_consume_char(|x| x == b'E' || x == b'e')?
            .is_some()
        {
            slice.try_consume_char(|x| x == b'-' || x == b'+')?;
            slice
                .try_consume_digit()?
                .ok_or(JsonError::UnexpectedInput)?;
            while slice.try_consume_digit()?.is_some() {}
        }
        let result = std::str::from_utf8(slice.end()?)?;
        let result = result.parse()?;
        Ok(result)
    }
}
