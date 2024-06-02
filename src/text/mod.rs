use crate::text::error::{TextError, TextResult};
use std::char::decode_utf16;

mod any;
mod bomb;
mod depth_budget;
mod error;
mod map;
mod number;
mod seq;
mod test;
mod value;

pub struct TextParser<'de> {
    cursor: &'de [u8],
    poisoned: bool,
}

impl<'de> TextParser<'de> {
    pub fn new(input: &'de [u8]) -> Self {
        TextParser {
            cursor: input,
            poisoned: false,
        }
    }
}

impl<'de> TextParser<'de> {
    pub fn try_peek_char(&self) -> TextResult<Option<u8>> {
        if self.poisoned {
            return Err(TextError::Poisoned);
        }
        Ok(self.cursor.get(0).cloned())
    }
    pub fn peek_char(&self) -> TextResult<u8> {
        self.try_peek_char()?.ok_or(TextError::Eof)
    }
    pub fn try_peek_ahead(&self, n: usize) -> TextResult<Option<u8>> {
        if self.poisoned {
            return Err(TextError::Poisoned);
        }
        Ok(self.cursor.get(n).cloned())
    }
    pub fn peek_count(&self, count: usize) -> TextResult<&'de [u8]> {
        self.cursor.get(..count).ok_or(TextError::Eof)
    }
    pub fn try_read_char(&mut self) -> TextResult<Option<u8>> {
        if self.poisoned {
            return Err(TextError::Poisoned);
        }
        if let Some(a) = self.cursor.take(..1) {
            Ok(Some(a[0]))
        } else {
            Ok(None)
        }
    }
    pub fn read_char(&mut self) -> TextResult<u8> {
        self.try_read_char()?.ok_or(TextError::Eof)
    }
    pub fn consume_char(&mut self, expected: impl FnOnce(u8) -> bool) -> TextResult<bool> {
        if self.poisoned {
            return Err(TextError::Poisoned);
        }
        if let Some((a, b)) = self.cursor.split_at_checked(1) {
            if expected(a[0]) {
                self.cursor = b;
                Ok(true)
            } else {
                Ok(false)
            }
        } else {
            Err(TextError::Eof)
        }
    }
    pub fn read_unicode(&mut self) -> TextResult<char> {
        let first = self.peek_char()?;
        let slice = self.consume_count(utf8_width::get_width(first))?;
        let c = std::str::from_utf8(slice)?
            .chars()
            .next()
            .ok_or(TextError::Utf8Error)?;
        Ok(c)
    }
    pub fn consume_slice(&mut self, expected: impl Fn(u8) -> bool) -> TextResult<&'de [u8]> {
        if self.poisoned {
            return Err(TextError::Poisoned);
        }
        let limit = self
            .cursor
            .iter()
            .position(|x| !expected(*x))
            .unwrap_or(self.cursor.len());
        Ok(self.cursor.take(..limit).unwrap())
    }
    pub fn consume_count(&mut self, count: usize) -> TextResult<&'de [u8]> {
        self.cursor.take(..count).ok_or(TextError::Eof)
    }
    pub fn read_whitespace(&mut self) -> TextResult<()> {
        self.consume_slice(|x| matches!(x, b' ' | b'\n' | b'\r' | b'\t'))?;
        Ok(())
    }
    pub fn try_consume_token(&mut self, expected: u8) -> TextResult<bool> {
        self.read_whitespace()?;
        if !self.consume_char(|x| x == expected)? {
            return Ok(false);
        }
        Ok(true)
    }
    pub fn consume_token(&mut self, expected: u8) -> TextResult<()> {
        self.read_whitespace()?;
        if !self.consume_char(|x| x == expected)? {
            return Err(TextError::ExpectedToken {
                expected: char::from(expected),
                found: self.cursor.get(0).map(|x| char::from(*x)),
            });
        }
        Ok(())
    }
    pub fn read_hex_u16(&mut self) -> TextResult<u16> {
        Ok(u16::from_str_radix(
            std::str::from_utf8(self.consume_count(4)?)?,
            16,
        )?)
    }
    pub fn read_string(&mut self) -> TextResult<String> {
        self.consume_token(b'"')?;
        let mut result = String::new();
        loop {
            let c = self.read_unicode()?;
            if c as u32 <= 0x1F {
                return Err(TextError::StringContainsControl);
            }
            match c {
                '"' => break,
                '\\' => {
                    let escaped = self.read_char()?;
                    let escaped: char = match escaped {
                        b'"' => '"',
                        b'\\' => '\\',
                        b'/' => '/',
                        b'b' => char::from(8),
                        b'f' => char::from(12),
                        b'n' => '\n',
                        b'r' => '\r',
                        b't' => '\t',
                        b'u' => {
                            let n1 = self.read_hex_u16()?;
                            if n1.is_utf16_surrogate() {
                                self.consume_token(b'\\')?;
                                self.consume_token(b'u')?;
                                let n2 = self.read_hex_u16()?;
                                decode_utf16([n1, n2])
                                    .next()
                                    .ok_or(TextError::StringBadEscape)??
                            } else {
                                char::try_from(n1 as u32)?
                            }
                        }
                        _ => return Err(TextError::StringBadEscape),
                    };
                    result.push(escaped);
                }
                x => result.push(x),
            }
        }
        Ok(result)
    }
    pub fn end_parsing(&mut self) -> TextResult<()> {
        self.read_whitespace()?;
        if !self.cursor.is_empty() {
            Err(TextError::TrailingText)
        } else {
            Ok(())
        }
    }
}
