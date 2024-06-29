use marshal_core::{Primitive, PrimitiveType};
use rc_slice2::ArcSlice;

use crate::decode::error::JsonDecoderError;
use crate::decode::SimpleJsonDecoder;

impl SimpleJsonDecoder {
    pub fn try_peek_char(&self) -> anyhow::Result<Option<u8>> {
        Ok(self.cursor.get(0).cloned())
    }
    pub fn peek_char(&self) -> anyhow::Result<u8> {
        Ok(self.try_peek_char()?.ok_or(JsonDecoderError::Eof)?)
    }
    pub fn try_peek_ahead(&self, n: usize) -> anyhow::Result<Option<u8>> {
        Ok(self.cursor.get(n).cloned())
    }
    pub fn peek_count(&self, count: usize) -> anyhow::Result<&[u8]> {
        Ok(self.cursor.get(..count).ok_or(JsonDecoderError::Eof)?)
    }
    pub fn try_read_char(&mut self) -> anyhow::Result<Option<u8>> {
        todo!();
        // if let Some(a) = self.cursor.take(..1) {
        //     Ok(Some(a[0]))
        // } else {
        //     Ok(None)
        // }
    }
    pub fn read_char(&mut self) -> anyhow::Result<u8> {
        Ok(self.try_read_char()?.ok_or(JsonDecoderError::Eof)?)
    }
    pub fn try_read_match(&mut self, expected: impl FnOnce(u8) -> bool) -> anyhow::Result<bool> {
        if let Some(head) = self.cursor.first() {
            if expected(*head) {
                ArcSlice::advance(&mut self.cursor, 1);
                Ok(true)
            } else {
                Ok(false)
            }
        } else {
            Err(JsonDecoderError::Eof.into())
        }
    }
    pub fn read_matches(&mut self, expected: impl Fn(u8) -> bool) -> anyhow::Result<&[u8]> {
        let limit = self
            .cursor
            .iter()
            .position(|x| !expected(*x))
            .unwrap_or(self.cursor.len());
        self.read_count(limit)
    }
    pub fn read_count(&mut self, count: usize) -> anyhow::Result<&[u8]> {
        Ok(ArcSlice::advance(&mut self.cursor, count).ok_or(JsonDecoderError::Eof)?)
    }
    pub fn read_whitespace(&mut self) -> anyhow::Result<()> {
        self.read_matches(|x| matches!(x, b' ' | b'\n' | b'\r' | b'\t'))?;
        Ok(())
    }
    pub fn try_read_exact(&mut self, expected: u8) -> anyhow::Result<bool> {
        self.read_whitespace()?;
        if !self.try_read_match(|x| x == expected)? {
            return Ok(false);
        }
        Ok(true)
    }
    pub fn read_exact(&mut self, expected: u8) -> anyhow::Result<()> {
        self.read_whitespace()?;
        if !self.try_read_match(|x| x == expected)? {
            return Err(JsonDecoderError::ExpectedToken {
                expected: char::from(expected),
                found: self.cursor.get(0).map(|x| char::from(*x)),
            }
            .into());
        }
        Ok(())
    }

    pub fn read_token(&mut self) -> anyhow::Result<&[u8]> {
        self.read_whitespace()?;
        Ok(self.read_matches(|x| x.is_ascii_alphabetic())?)
    }

    pub fn read_bool(&mut self) -> anyhow::Result<bool> {
        match self.read_token()? {
            b"false" => Ok(false),
            b"true" => Ok(true),
            x => Err(JsonDecoderError::UnexpectedIdentifier { found: x.to_vec() }.into()),
        }
    }

    pub fn read_null(&mut self) -> anyhow::Result<()> {
        match self.read_token()? {
            b"null" => Ok(()),
            x => Err(JsonDecoderError::UnexpectedIdentifier { found: x.to_vec() }.into()),
        }
    }

    pub fn read_prim_from_str(&mut self, prim: PrimitiveType) -> anyhow::Result<Primitive> {
        self.read_exact(b'\"')?;
        let result = match prim {
            PrimitiveType::Unit => Primitive::Unit,
            PrimitiveType::Bool => Primitive::Bool(self.read_bool()?),
            PrimitiveType::I8 => Primitive::I8(self.read_number()?),
            PrimitiveType::I16 => Primitive::I16(self.read_number()?),
            PrimitiveType::I32 => Primitive::I32(self.read_number()?),
            PrimitiveType::I64 => Primitive::I64(self.read_number()?),
            PrimitiveType::I128 => Primitive::I128(self.read_number()?),
            PrimitiveType::U8 => Primitive::U8(self.read_number()?),
            PrimitiveType::U16 => Primitive::U16(self.read_number()?),
            PrimitiveType::U32 => Primitive::U32(self.read_number()?),
            PrimitiveType::U64 => Primitive::U64(self.read_number()?),
            PrimitiveType::U128 => Primitive::U128(self.read_number()?),
            PrimitiveType::F32 => Primitive::F32(self.read_number()?),
            PrimitiveType::F64 => Primitive::F64(self.read_number()?),
            PrimitiveType::Char => unreachable!(),
        };
        self.read_exact(b'\"')?;
        Ok(result)
    }

    pub fn end(mut self) -> anyhow::Result<()> {
        self.read_whitespace()?;
        if !self.cursor.is_empty() {
            Err(JsonDecoderError::TrailingText.into())
        } else {
            Ok(())
        }
    }
}
