use std::str::FromStr;

use crate::text::bomb::TextBomb;
use crate::text::error::{TextError, TextResult};
use crate::text::TextParser;

pub struct TextNumberParser<'p, 'de> {
    parser: TextBomb<'p, 'de>,
}

struct SliceParser<'p, 'de> {
    parser: TextBomb<'p, 'de>,
    index: usize,
}

impl<'p, 'de> SliceParser<'p, 'de> {
    pub fn new(parser: &'p mut TextParser<'de>) -> Self {
        SliceParser {
            parser: TextBomb::new(parser),
            index: 0,
        }
    }
    pub fn try_consume_char(
        &mut self,
        expected: impl FnOnce(u8) -> bool,
    ) -> TextResult<Option<u8>> {
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
    pub fn try_consume_digit(&mut self) -> TextResult<Option<u8>> {
        Ok(self.try_consume_char(|x| x.is_ascii_digit())?)
    }

    pub fn end(self) -> TextResult<&'de [u8]> {
        self.parser.into_inner().consume_count(self.index)
    }
}

impl<'p, 'de> TextNumberParser<'p, 'de> {
    pub fn new(parser: &'p mut TextParser<'de>) -> Self {
        TextNumberParser {
            parser: TextBomb::new(parser),
        }
    }
    pub fn parse_number<T: FromStr>(mut self) -> TextResult<T>
    where
        TextError: From<T::Err>,
    {
        let mut slice = SliceParser::new(&mut *self.parser);
        slice.try_consume_char(|x| x == b'-')?;
        match slice
            .try_consume_digit()?
            .ok_or(TextError::UnexpectedInput)?
        {
            b'0' => {}
            _ => while slice.try_consume_digit()?.is_some() {},
        }
        if slice.try_consume_char(|x| x == b'.')?.is_some() {
            slice
                .try_consume_digit()?
                .ok_or(TextError::UnexpectedInput)?;
            while slice.try_consume_digit()?.is_some() {}
        }
        if slice
            .try_consume_char(|x| x == b'E' || x == b'e')?
            .is_some()
        {
            slice.try_consume_char(|x| x == b'-' || x == b'+')?;
            slice
                .try_consume_digit()?
                .ok_or(TextError::UnexpectedInput)?;
            while slice.try_consume_digit()?.is_some() {}
        }
        let result = std::str::from_utf8(slice.end()?)?;
        let result = result.parse()?;
        self.parser.defuse();
        Ok(result)
    }
}
