use std::char::decode_utf16;
use crate::error::ParseResult;

use crate::json::error::JsonError;
use crate::json::JsonParser;

impl<'de> JsonParser<'de> {
    pub fn read_hex_u16(&mut self) -> ParseResult<u16> {
        Ok(u16::from_str_radix(
            std::str::from_utf8(self.read_count(4)?)?,
            16,
        )?)
    }
    pub fn read_string(&mut self) -> ParseResult<String> {
        self.read_exact(b'"')?;
        let mut result = String::new();
        loop {
            let c = self.read_unicode()?;
            if c as u32 <= 0x1F {
                return Err(JsonError::StringContainsControl.into());
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
                                self.read_exact(b'\\')?;
                                self.read_exact(b'u')?;
                                let n2 = self.read_hex_u16()?;
                                decode_utf16([n1, n2])
                                    .next()
                                    .ok_or(JsonError::StringBadEscape)??
                            } else {
                                char::try_from(n1 as u32)?
                            }
                        }
                        _ => return Err(JsonError::StringBadEscape.into()),
                    };
                    result.push(escaped);
                }
                x => result.push(x),
            }
        }
        Ok(result)
    }
    pub fn read_unicode(&mut self) -> ParseResult<char> {
        let first = self.peek_char()?;
        let slice = self.read_count(utf8_width::get_width(first))?;
        let c = std::str::from_utf8(slice)?
            .chars()
            .next()
            .ok_or(JsonError::Utf8Error)?;
        Ok(c)
    }
}
