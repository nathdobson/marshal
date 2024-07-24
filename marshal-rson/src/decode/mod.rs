use crate::RsonError;
use anyhow;
use base64::prelude::BASE64_STANDARD;
use base64::Engine;
use marshal::decode::{DecodeHint, DecodeVariantHint, SimpleDecoderView, SpecDecoder};
use marshal::Primitive;
use std::borrow::Cow;

pub mod full;

pub struct SimpleRsonSpecDecoder<'de> {
    original: &'de str,
    cursor: &'de str,
}

impl<'de> SimpleRsonSpecDecoder<'de> {
    pub fn new(data: &'de str) -> Self {
        SimpleRsonSpecDecoder {
            original: data,
            cursor: data,
        }
    }
    pub fn location(&self) -> String {
        let offset = self.original.len() - self.cursor.len();
        let consumed = &self.original[0..offset];
        let mut line = 0;
        let mut column = 0;
        for c in consumed.chars() {
            if c == '\n' {
                line += 1;
                column = 0;
            } else {
                column += 1;
            }
        }
        format!("at line {} column {}", line, column)
    }
    pub fn read_bytes(&mut self, count: usize) -> anyhow::Result<&'de str> {
        let (a, b) = self
            .cursor
            .split_at_checked(count)
            .ok_or(RsonError::UnexpectedEof)?;
        self.cursor = b;
        Ok(a)
    }

    pub fn read_char(&mut self) -> anyhow::Result<char> {
        let mut chars = self.cursor.chars();
        let c = chars.next().ok_or(RsonError::UnexpectedEof)?;
        self.cursor = chars.as_str();
        Ok(c)
    }
    pub fn read_matches(&mut self, expected: impl Fn(char) -> bool) -> anyhow::Result<&'de str> {
        let mut chars = self.cursor.char_indices();
        let mut limit = self.cursor.len();
        while let Some((pos, c)) = chars.next() {
            if !expected(c) {
                limit = pos;
                break;
            }
        }
        self.read_bytes(limit)
    }
    pub fn read_whitespace(&mut self) -> anyhow::Result<()> {
        self.read_matches(|x| x.is_whitespace())?;
        Ok(())
    }
    pub fn try_read_eof(&mut self) -> anyhow::Result<bool> {
        self.read_whitespace()?;
        Ok(self.cursor.len() == 0)
    }
    pub fn try_read_token(&mut self, token: &str) -> anyhow::Result<bool> {
        self.read_whitespace()?;
        if self.cursor.starts_with(token) {
            self.read_bytes(token.len())?;
            Ok(true)
        } else {
            Ok(false)
        }
    }
    pub fn read_token(&mut self, token: &'static str) -> anyhow::Result<()> {
        if !self.try_read_token(token)? {
            Err(RsonError::ExpectedToken { token }.into())
        } else {
            Ok(())
        }
    }
    pub fn try_read_alphanum(&mut self) -> anyhow::Result<Option<&'de str>> {
        self.read_whitespace()?;
        let ident = self.read_matches(|x| {
            !x.is_whitespace()
                && !matches!(
                    x,
                    ',' | ':' | '{' | '}' | '[' | ']' | '(' | ')' | '"' | '\''
                )
        })?;
        if ident.len() > 0 {
            Ok(Some(ident))
        } else {
            Ok(None)
        }
    }
    pub fn end(self) -> anyhow::Result<()> {
        if self.cursor.chars().any(|x| !x.is_whitespace()) {
            return Err(RsonError::TrailingData.into());
        }
        Ok(())
    }
}

pub enum RsonAnyDecoder<'de> {
    Any,
    String(&'de str),
}

impl<'de> RsonAnyDecoder<'de> {
    pub fn new() -> Self {
        RsonAnyDecoder::Any
    }
}

pub struct RsonSeqDecoder {
    started: bool,
    terminal: &'static str,
}

pub struct RsonMapDecoder {
    started: bool,
    is_struct: bool,
}

pub struct RsonKeyDecoder {
    is_struct: bool,
}

pub struct RsonDiscriminantDecoder<'de> {
    variant: &'de str,
}

impl<'de> SpecDecoder<'de> for SimpleRsonSpecDecoder<'de> {
    type AnyDecoder = RsonAnyDecoder<'de>;
    type SeqDecoder = RsonSeqDecoder;
    type MapDecoder = RsonMapDecoder;
    type KeyDecoder = RsonKeyDecoder;
    type ValueDecoder = ();
    type DiscriminantDecoder = RsonDiscriminantDecoder<'de>;
    type VariantDecoder = ();
    type EnumCloser = ();
    type SomeDecoder = ();
    type SomeCloser = ();

    fn decode(
        &mut self,
        any: Self::AnyDecoder,
        hint: DecodeHint,
    ) -> anyhow::Result<SimpleDecoderView<'de, Self>> {
        match any {
            RsonAnyDecoder::Any => {}
            RsonAnyDecoder::String(x) => return Ok(SimpleDecoderView::String(Cow::Borrowed(x))),
        }
        if self.try_read_token("{")? {
            Ok(SimpleDecoderView::Map(RsonMapDecoder {
                started: false,
                is_struct: false,
            }))
        } else if self.try_read_token("[")? {
            Ok(SimpleDecoderView::Seq(RsonSeqDecoder {
                started: false,
                terminal: "]",
            }))
        } else if self.try_read_token("(")? {
            if self.try_read_token(")")? {
                Ok(SimpleDecoderView::Primitive(Primitive::Unit))
            } else {
                Ok(SimpleDecoderView::Seq(RsonSeqDecoder {
                    started: false,
                    terminal: ")",
                }))
            }
        } else if let Some(ident) = self.try_read_alphanum()? {
            match ident {
                "unit" => Ok(SimpleDecoderView::Primitive(Primitive::Unit)),
                "false" => Ok(SimpleDecoderView::Primitive(Primitive::Bool(false))),
                "true" => Ok(SimpleDecoderView::Primitive(Primitive::Bool(true))),
                "i8" | "i16" | "i32" | "i64" | "i128" | "u8" | "u16" | "u32" | "u64" | "u128"
                | "f32" | "f64" => {
                    let arg = self.try_read_alphanum()?.ok_or(RsonError::ExpectedNumber)?;
                    match ident {
                        "i8" => Ok(SimpleDecoderView::Primitive(Primitive::I8(arg.parse()?))),
                        "i16" => Ok(SimpleDecoderView::Primitive(Primitive::I16(arg.parse()?))),
                        "i32" => Ok(SimpleDecoderView::Primitive(Primitive::I32(arg.parse()?))),
                        "i64" => Ok(SimpleDecoderView::Primitive(Primitive::I64(arg.parse()?))),
                        "i128" => Ok(SimpleDecoderView::Primitive(Primitive::I128(arg.parse()?))),
                        "u8" => Ok(SimpleDecoderView::Primitive(Primitive::U8(arg.parse()?))),
                        "u16" => Ok(SimpleDecoderView::Primitive(Primitive::U16(arg.parse()?))),
                        "u32" => Ok(SimpleDecoderView::Primitive(Primitive::U32(arg.parse()?))),
                        "u64" => Ok(SimpleDecoderView::Primitive(Primitive::U64(arg.parse()?))),
                        "u128" => Ok(SimpleDecoderView::Primitive(Primitive::U128(arg.parse()?))),
                        "f32" => Ok(SimpleDecoderView::Primitive(Primitive::F32(arg.parse()?))),
                        "f64" => Ok(SimpleDecoderView::Primitive(Primitive::F64(arg.parse()?))),
                        _ => todo!(),
                    }
                }
                "char" => {
                    self.read_token("'")?;
                    let result = SimpleDecoderView::Primitive(Primitive::Char(self.read_char()?));
                    self.read_token("'")?;
                    Ok(result)
                }
                "string" => {
                    self.read_token("\"")?;
                    let mut result = String::new();
                    let mut chars = self.cursor.chars();
                    loop {
                        let c = chars.next().ok_or(RsonError::UnexpectedEof)?;
                        match c {
                            '\"' => break,
                            '\\' => match chars.next().ok_or(RsonError::UnexpectedEof)? {
                                'n' => result.push('\n'),
                                '\\' => result.push('\\'),
                                '"' => result.push('"'),
                                _ => return Err(RsonError::UnexpectedEscape.into()),
                            },
                            _ => result.push(c),
                        }
                    }
                    self.cursor = chars.as_str();
                    Ok(SimpleDecoderView::String(Cow::Owned(result)))
                }
                "bytes" => {
                    self.read_token("\"")?;
                    let encoded =
                        self.read_matches(|x| x.is_ascii_alphanumeric() || x == '+' || x == '/')?;
                    let result = BASE64_STANDARD.decode(encoded.as_bytes())?;
                    self.read_token("\"")?;
                    Ok(SimpleDecoderView::Bytes(Cow::Owned(result)))
                }
                "struct" => {
                    let _name = self.try_read_alphanum()?.ok_or(RsonError::ExpectedIdent)?;
                    if self.try_read_token("(")? {
                        Ok(SimpleDecoderView::Seq(RsonSeqDecoder {
                            started: false,
                            terminal: ")",
                        }))
                    } else if self.try_read_token("{")? {
                        Ok(SimpleDecoderView::Map(RsonMapDecoder {
                            started: false,
                            is_struct: true,
                        }))
                    } else {
                        Ok(SimpleDecoderView::Primitive(Primitive::Unit))
                    }
                }
                "enum" => {
                    let _name = self.try_read_alphanum()?.ok_or(RsonError::ExpectedIdent)?;
                    self.read_token("::")?;
                    let variant = self.try_read_alphanum()?.ok_or(RsonError::ExpectedIdent)?;
                    Ok(SimpleDecoderView::Enum(RsonDiscriminantDecoder { variant }))
                }
                "none" => Ok(SimpleDecoderView::None),
                "some" => Ok(SimpleDecoderView::Some(())),
                _ => Err(RsonError::UnexpectedKind {
                    kind: ident.to_owned(),
                }
                .into()),
            }
        } else {
            println!("{:?} {:?}", self.cursor, hint);
            todo!();
        }
    }

    fn is_human_readable(&self) -> bool {
        todo!()
    }

    fn decode_seq_next(
        &mut self,
        seq: &mut Self::SeqDecoder,
    ) -> anyhow::Result<Option<Self::AnyDecoder>> {
        if seq.started {
            if !self.try_read_token(",")? {
                self.read_token(seq.terminal)?;
                return Ok(None);
            }
        }
        if self.try_read_token(seq.terminal)? {
            return Ok(None);
        }
        seq.started = true;
        Ok(Some(RsonAnyDecoder::Any))
    }

    fn decode_seq_end(&mut self, _seq: Self::SeqDecoder) -> anyhow::Result<()> {
        Ok(())
    }

    fn decode_map_next(
        &mut self,
        map: &mut Self::MapDecoder,
    ) -> anyhow::Result<Option<Self::KeyDecoder>> {
        if map.started {
            if !self.try_read_token(",")? {
                self.read_token("}")?;
                return Ok(None);
            }
        }
        if self.try_read_token("}")? {
            return Ok(None);
        }
        map.started = true;
        Ok(Some(RsonKeyDecoder {
            is_struct: map.is_struct,
        }))
    }

    fn decode_map_end(&mut self, _seq: Self::MapDecoder) -> anyhow::Result<()> {
        Ok(())
    }

    fn decode_entry_key(
        &mut self,
        key: Self::KeyDecoder,
    ) -> anyhow::Result<(Self::AnyDecoder, Self::ValueDecoder)> {
        if key.is_struct {
            let name = self.try_read_alphanum()?.ok_or(RsonError::ExpectedIdent)?;
            Ok((RsonAnyDecoder::String(name), ()))
        } else {
            Ok((RsonAnyDecoder::Any, ()))
        }
    }

    fn decode_entry_value(
        &mut self,
        _value: Self::ValueDecoder,
    ) -> anyhow::Result<Self::AnyDecoder> {
        self.read_token(":")?;
        Ok(RsonAnyDecoder::Any)
    }

    fn decode_enum_discriminant(
        &mut self,
        e: Self::DiscriminantDecoder,
    ) -> anyhow::Result<(Self::AnyDecoder, Self::VariantDecoder)> {
        Ok((RsonAnyDecoder::String(e.variant), ()))
    }

    fn decode_enum_variant(
        &mut self,
        _e: Self::VariantDecoder,
        _hint: DecodeVariantHint,
    ) -> anyhow::Result<(SimpleDecoderView<'de, Self>, Self::EnumCloser)> {
        if self.try_read_token("{")? {
            Ok((
                SimpleDecoderView::Map(RsonMapDecoder {
                    started: false,
                    is_struct: true,
                }),
                (),
            ))
        } else if self.try_read_token("(")? {
            Ok((
                SimpleDecoderView::Seq(RsonSeqDecoder {
                    started: false,
                    terminal: ")",
                }),
                (),
            ))
        } else {
            Ok((SimpleDecoderView::Primitive(Primitive::Unit), ()))
        }
    }

    fn decode_enum_end(&mut self, _e: Self::EnumCloser) -> anyhow::Result<()> {
        Ok(())
    }

    fn decode_some_inner(
        &mut self,
        _e: Self::SomeDecoder,
    ) -> anyhow::Result<(Self::AnyDecoder, Self::SomeCloser)> {
        Ok((RsonAnyDecoder::Any, ()))
    }

    fn decode_some_end(&mut self, _p: Self::SomeCloser) -> anyhow::Result<()> {
        Ok(())
    }
}
