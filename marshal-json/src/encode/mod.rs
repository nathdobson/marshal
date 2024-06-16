use std::error::Error;
use std::fmt;
use std::fmt::{Display, Formatter};
use std::io::Write;

use base64::Engine;
use base64::prelude::BASE64_STANDARD_NO_PAD;

use marshal_core::encode::simple::{SimpleAnyEncoder, SimpleEncoder};
use marshal_core::Primitive;

pub mod full;
#[cfg(test)]
mod test;

pub struct SimpleJsonEncoder {
    output: Vec<u8>,
    current_indentation: Option<usize>,
}

#[derive(Debug)]
pub enum JsonEncoderError {
    BadNumber,
    NumericOverflow,
}

impl Display for JsonEncoderError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "JsonEncoderError")
    }
}

impl Error for JsonEncoderError {}

impl SimpleJsonEncoder {
    pub fn new() -> Self {
        SimpleJsonEncoder {
            output: vec![],
            current_indentation: Some(0),
        }
    }
    pub fn start(&mut self) -> SimpleAnyEncoder<SimpleJsonEncoder> {
        SimpleAnyEncoder::new(
            self,
            JsonAnyEncoder {
                ctx: EncodeContext { indentation: 0 },
                must_be_string: false,
                cannot_be_null: false,
            },
        )
    }
    fn set_indentation(&mut self, indentation: usize) -> anyhow::Result<()> {
        if let Some(current) = self.current_indentation {
            if current != indentation {
                writeln!(&mut self.output)?;
                self.current_indentation = None;
            }
        }
        if self.current_indentation.is_none() {
            for _ in 0..indentation * 2 {
                write!(&mut self.output, " ")?;
            }
            self.current_indentation = Some(indentation);
        }
        Ok(())
    }
    pub fn end(self) -> anyhow::Result<String> {
        Ok(String::from_utf8(self.output)?)
    }

    fn write(&mut self, ctx: EncodeContext, value: impl Display) -> anyhow::Result<()> {
        self.set_indentation(ctx.indentation)?;
        write!(&mut self.output, "{}", value)?;
        Ok(())
    }
    fn writeln(&mut self, ctx: EncodeContext, value: impl Display) -> anyhow::Result<()> {
        self.set_indentation(ctx.indentation)?;
        write!(&mut self.output, "{}\n", value)?;
        self.current_indentation = None;
        Ok(())
    }
    fn write_null(&mut self, ctx: EncodeContext) -> anyhow::Result<()> {
        self.write(ctx, "null")
    }
    fn open_map(&mut self, ctx: EncodeContext) -> anyhow::Result<()> {
        self.write(ctx, "{")
    }
    fn close_map(&mut self, ctx: EncodeContext) -> anyhow::Result<()> {
        self.write(ctx, "}")
    }
    fn open_list(&mut self, ctx: EncodeContext) -> anyhow::Result<()> {
        self.write(ctx, "[")
    }
    fn close_list(&mut self, ctx: EncodeContext) -> anyhow::Result<()> {
        self.write(ctx, "]")
    }
    fn write_str_literal(&mut self, ctx: EncodeContext, s: &str) -> anyhow::Result<()> {
        self.write(ctx, "\"")?;
        for c in s.chars() {
            match c {
                '\\' => write!(&mut self.output, "\\\\")?,
                '"' => write!(&mut self.output, "\\\"")?,
                '\u{0008}' => write!(&mut self.output, "\\b")?,
                '\u{000c}' => write!(&mut self.output, "\\f")?,
                '\n' => write!(&mut self.output, "\\n")?,
                '\r' => write!(&mut self.output, "\\r")?,
                '\t' => write!(&mut self.output, "\\t")?,
                ..='\u{001f}' => {
                    write!(&mut self.output, "\\u{:0>4x}", c as u32)?;
                }
                _ => write!(&mut self.output, "{}", c)?,
            }
        }
        self.write(ctx, "\"")?;
        Ok(())
    }
    fn write_colon(&mut self, ctx: EncodeContext) -> anyhow::Result<()> {
        self.write(ctx, ": ")
    }
    fn write_comma(&mut self, ctx: EncodeContext) -> anyhow::Result<()> {
        self.writeln(ctx, ",")
    }
}

impl SimpleEncoder for SimpleJsonEncoder {
    type AnyEncoder = JsonAnyEncoder;
    type SomeCloser = JsonSomeCloser;
    type TupleEncoder = JsonTupleEncoder;
    type SeqEncoder = JsonSeqEncoder;
    type MapEncoder = JsonMapEncoder;
    type ValueEncoder = JsonValueEncoder;
    type EntryCloser = JsonEntryEncoder;
    type TupleStructEncoder = JsonTupleStructEncoder;
    type StructEncoder = JsonStructEncoder;
    type TupleVariantEncoder = JsonTupleVariantEncoder;
    type StructVariantEncoder = JsonStructVariantEncoder;

    fn encode_prim(&mut self, any: Self::AnyEncoder, prim: Primitive) -> anyhow::Result<()> {
        match prim {
            Primitive::Unit => self.write_null(any.ctx),
            Primitive::Bool(x) => self.write(any.ctx, x),
            Primitive::I8(x) => self.write(any.ctx, x),
            Primitive::I16(x) => self.write(any.ctx, x),
            Primitive::I32(x) => self.write(any.ctx, x),
            Primitive::I64(x) => self.write(any.ctx, x),
            Primitive::I128(x) => self.write(any.ctx, x),
            Primitive::U8(x) => self.write(any.ctx, x),
            Primitive::U16(x) => self.write(any.ctx, x),
            Primitive::U32(x) => self.write(any.ctx, x),
            Primitive::U64(x) => self.write(any.ctx, x),
            Primitive::U128(x) => self.write(any.ctx, x),
            Primitive::F32(x) => {
                if x.is_finite() {
                    self.write(any.ctx, x)
                } else {
                    return Err(JsonEncoderError::BadNumber)?;
                }
            }
            Primitive::F64(x) => {
                if x.is_finite() {
                    self.write(any.ctx, x)
                } else {
                    return Err(JsonEncoderError::BadNumber)?;
                }
            }
            Primitive::Char(x) => self.write_str_literal(any.ctx, x.encode_utf8(&mut [0u8; 4])),
        }
    }

    fn encode_str(&mut self, any: Self::AnyEncoder, s: &str) -> anyhow::Result<()> {
        self.write_str_literal(any.ctx, s)
    }

    fn encode_bytes(&mut self, any: Self::AnyEncoder, s: &[u8]) -> anyhow::Result<()> {
        self.write(any.ctx, "\"")?;
        let len = base64::encoded_len(s.len(), false).ok_or(JsonEncoderError::NumericOverflow)?;
        let start = self.output.len();
        self.output.resize(start + len, 0);
        BASE64_STANDARD_NO_PAD.encode(&mut self.output[start..]);
        self.write(any.ctx, "\"")?;
        Ok(())
    }

    fn encode_none(&mut self, any: Self::AnyEncoder) -> anyhow::Result<()> {
        if any.cannot_be_null {
            self.open_map(any.ctx)?;
            let ctx = any.ctx.indent();
            self.write_str_literal(ctx, "None")?;
            self.write_colon(ctx)?;
            self.write_null(ctx)?;
            self.close_map(any.ctx)?;
        } else {
            self.write_null(any.ctx)?;
        }
        Ok(())
    }

    fn encode_some(
        &mut self,
        any: Self::AnyEncoder,
    ) -> anyhow::Result<(Self::AnyEncoder, Self::SomeCloser)> {
        if any.cannot_be_null {
            self.open_map(any.ctx)?;
            let ctx = any.ctx.indent();
            self.write_str_literal(ctx, "Some")?;
            self.write_colon(ctx)?;
            Ok((
                JsonAnyEncoder {
                    ctx: ctx,
                    must_be_string: false,
                    cannot_be_null: false,
                },
                JsonSomeCloser {
                    ctx: any.ctx,
                    cannot_be_null: true,
                },
            ))
        } else {
            Ok((
                JsonAnyEncoder {
                    ctx: any.ctx,
                    must_be_string: any.must_be_string,
                    cannot_be_null: true,
                },
                JsonSomeCloser {
                    ctx: any.ctx,
                    cannot_be_null: false,
                },
            ))
        }
    }

    fn encode_unit_struct(&mut self, any: Self::AnyEncoder, _: &'static str) -> anyhow::Result<()> {
        self.write_null(any.ctx)
    }

    fn encode_tuple_struct(
        &mut self,
        any: Self::AnyEncoder,
        _: &'static str,
        _: usize,
    ) -> anyhow::Result<Self::TupleStructEncoder> {
        self.open_list(any.ctx)?;
        Ok(JsonTupleStructEncoder {
            ctx: any.ctx,
            started: false,
        })
    }

    fn encode_struct(
        &mut self,
        any: Self::AnyEncoder,
        _: &'static str,
        _: &'static [&'static str],
    ) -> anyhow::Result<Self::StructEncoder> {
        self.open_map(any.ctx)?;
        Ok(JsonStructEncoder {
            ctx: any.ctx,
            started: false,
        })
    }

    fn encode_unit_variant(
        &mut self,
        any: Self::AnyEncoder,
        _name: &'static str,
        variants: &'static [&'static str],
        variant_index: usize,
    ) -> anyhow::Result<()> {
        self.open_map(any.ctx)?;
        let ctx = any.ctx.indent();
        self.write_str_literal(ctx, variants[variant_index])?;
        self.write_colon(ctx)?;
        self.write_null(ctx)?;
        self.close_map(any.ctx)?;
        Ok(())
    }

    fn encode_tuple_variant(
        &mut self,
        any: Self::AnyEncoder,
        _name: &'static str,
        variants: &'static [&'static str],
        variant_index: usize,
        _len: usize,
    ) -> anyhow::Result<Self::TupleVariantEncoder> {
        self.open_map(any.ctx)?;
        let ctx = any.ctx.indent();
        self.write_str_literal(ctx, variants[variant_index])?;
        self.write_colon(ctx)?;
        self.open_list(ctx)?;
        Ok(JsonTupleVariantEncoder {
            ctx: any.ctx,
            started: false,
        })
    }

    fn encode_struct_variant(
        &mut self,
        any: Self::AnyEncoder,
        _name: &'static str,
        variants: &'static [&'static str],
        variant_index: usize,
        _fields: &'static [&'static str],
    ) -> anyhow::Result<Self::StructVariantEncoder> {
        self.open_map(any.ctx)?;
        let ctx = any.ctx.indent();
        self.write_str_literal(ctx, variants[variant_index])?;
        self.write_colon(ctx)?;
        self.open_map(ctx)?;
        Ok(JsonStructVariantEncoder {
            ctx: any.ctx,
            started: false,
        })
    }

    fn encode_seq(
        &mut self,
        any: Self::AnyEncoder,
        _len: Option<usize>,
    ) -> anyhow::Result<Self::SeqEncoder> {
        self.open_list(any.ctx)?;
        Ok(JsonSeqEncoder {
            ctx: any.ctx,
            started: false,
        })
    }

    fn encode_tuple(
        &mut self,
        any: Self::AnyEncoder,
        _len: usize,
    ) -> anyhow::Result<Self::TupleEncoder> {
        self.open_list(any.ctx)?;
        Ok(JsonTupleEncoder {
            ctx: any.ctx,
            started: false,
        })
    }

    fn encode_map(
        &mut self,
        any: Self::AnyEncoder,
        _len: Option<usize>,
    ) -> anyhow::Result<Self::MapEncoder> {
        self.open_map(any.ctx)?;
        Ok(JsonMapEncoder {
            ctx: any.ctx,
            started: false,
        })
    }

    fn some_end(&mut self, some: Self::SomeCloser) -> anyhow::Result<()> {
        if some.cannot_be_null {
            self.close_map(some.ctx)?;
        }
        Ok(())
    }

    fn tuple_encode_element(
        &mut self,
        tuple: &mut Self::TupleEncoder,
    ) -> anyhow::Result<Self::AnyEncoder> {
        let ctx = tuple.ctx.indent();
        if tuple.started {
            self.write_comma(ctx)?;
        }
        tuple.started = true;
        Ok(JsonAnyEncoder {
            ctx,
            must_be_string: false,
            cannot_be_null: false,
        })
    }

    fn tuple_end(&mut self, tuple: Self::TupleEncoder) -> anyhow::Result<()> {
        self.close_list(tuple.ctx)?;
        Ok(())
    }

    fn seq_encode_element(&mut self, seq: &mut Self::SeqEncoder) -> anyhow::Result<Self::AnyEncoder> {
        let ctx = seq.ctx.indent();
        if seq.started {
            self.write_comma(ctx)?;
        }
        seq.started = true;
        Ok(JsonAnyEncoder {
            ctx,
            must_be_string: false,
            cannot_be_null: false,
        })
    }

    fn seq_end(&mut self, tuple: Self::SeqEncoder) -> anyhow::Result<()> {
        self.close_list(tuple.ctx)
    }

    fn map_encode_element(
        &mut self,
        map: &mut Self::MapEncoder,
    ) -> anyhow::Result<(Self::AnyEncoder, Self::ValueEncoder)> {
        let ctx = map.ctx.indent();
        if map.started {
            self.write_comma(ctx)?;
        }
        map.started = true;
        Ok((
            JsonAnyEncoder {
                ctx,
                must_be_string: true,
                cannot_be_null: false,
            },
            JsonValueEncoder { ctx },
        ))
    }

    fn map_end(&mut self, map: Self::MapEncoder) -> anyhow::Result<()> {
        self.close_map(map.ctx)?;
        Ok(())
    }

    fn entry_encode_value(
        &mut self,
        value: Self::ValueEncoder,
    ) -> anyhow::Result<(Self::AnyEncoder, Self::EntryCloser)> {
        self.write_colon(value.ctx)?;
        Ok((
            JsonAnyEncoder {
                ctx: value.ctx,
                must_be_string: false,
                cannot_be_null: false,
            },
            JsonEntryEncoder { ctx: value.ctx },
        ))
    }

    fn entry_end(&mut self, _closer: Self::EntryCloser) -> anyhow::Result<()> {
        Ok(())
    }

    fn tuple_struct_encode_field(
        &mut self,
        tuple: &mut Self::TupleStructEncoder,
    ) -> anyhow::Result<Self::AnyEncoder> {
        let ctx = tuple.ctx.indent();
        if tuple.started {
            self.write_comma(ctx)?;
        }
        tuple.started = true;
        Ok(JsonAnyEncoder {
            ctx,
            must_be_string: false,
            cannot_be_null: false,
        })
    }

    fn tuple_struct_end(&mut self, tuple: Self::TupleStructEncoder) -> anyhow::Result<()> {
        self.close_list(tuple.ctx)?;
        Ok(())
    }

    fn struct_encode_field(
        &mut self,
        s: &mut Self::StructEncoder,
        key: &'static str,
    ) -> anyhow::Result<Self::AnyEncoder> {
        let ctx = s.ctx.indent();
        if s.started {
            self.write_comma(ctx)?;
        }
        s.started = true;
        self.write_str_literal(ctx, key)?;
        self.write_colon(ctx)?;
        Ok(JsonAnyEncoder {
            ctx,
            must_be_string: false,
            cannot_be_null: false,
        })
    }

    fn struct_end(&mut self, s: Self::StructEncoder) -> anyhow::Result<()> {
        self.close_map(s.ctx)?;
        Ok(())
    }

    fn tuple_variant_encode_field(
        &mut self,
        tuple: &mut Self::TupleVariantEncoder,
    ) -> anyhow::Result<Self::AnyEncoder> {
        let ctx = tuple.ctx.indent().indent();
        if tuple.started {
            self.write_comma(ctx)?;
        }
        tuple.started = true;
        Ok(JsonAnyEncoder {
            ctx,
            must_be_string: false,
            cannot_be_null: false,
        })
    }

    fn tuple_variant_end(&mut self, tuple: Self::TupleVariantEncoder) -> anyhow::Result<()> {
        self.close_list(tuple.ctx.indent())?;
        self.close_map(tuple.ctx)?;
        Ok(())
    }

    fn struct_variant_encode_field(
        &mut self,
        s: &mut Self::StructVariantEncoder,
        key: &'static str,
    ) -> anyhow::Result<Self::AnyEncoder> {
        let ctx = s.ctx.indent().indent();
        if s.started {
            self.write_comma(ctx)?;
        }
        s.started = true;
        self.write_str_literal(ctx, key)?;
        self.write_colon(ctx)?;
        Ok(JsonAnyEncoder {
            ctx,
            must_be_string: false,
            cannot_be_null: false,
        })
    }

    fn struct_variant_end(&mut self, s: Self::StructVariantEncoder) -> anyhow::Result<()> {
        self.close_map(s.ctx.indent())?;
        self.close_map(s.ctx)?;
        Ok(())
    }
}

#[derive(Copy, Clone)]
struct EncodeContext {
    indentation: usize,
}

impl EncodeContext {
    pub fn new() -> Self {
        EncodeContext { indentation: 0 }
    }
    pub fn indent(self) -> Self {
        EncodeContext {
            indentation: self.indentation + 1,
        }
    }
}

pub struct JsonAnyEncoder {
    ctx: EncodeContext,
    must_be_string: bool,
    cannot_be_null: bool,
}

impl JsonAnyEncoder {
    pub fn new() -> Self {
        JsonAnyEncoder {
            ctx: EncodeContext::new(),
            must_be_string: false,
            cannot_be_null: false,
        }
    }
}

pub struct JsonSomeEncoder {
    ctx: EncodeContext,
    must_be_string: bool,
    cannot_be_null: bool,
}

pub struct JsonSomeCloser {
    ctx: EncodeContext,
    cannot_be_null: bool,
}

pub struct JsonTupleEncoder {
    ctx: EncodeContext,
    started: bool,
}

pub struct JsonSeqEncoder {
    ctx: EncodeContext,
    started: bool,
}

pub struct JsonMapEncoder {
    ctx: EncodeContext,
    started: bool,
}

pub struct JsonKeyEncoder {
    ctx: EncodeContext,
}

pub struct JsonValueEncoder {
    ctx: EncodeContext,
}

pub struct JsonEntryEncoder {
    ctx: EncodeContext,
}

pub struct JsonTupleStructEncoder {
    ctx: EncodeContext,
    started: bool,
}

pub struct JsonStructEncoder {
    ctx: EncodeContext,
    started: bool,
}

pub struct JsonTupleVariantEncoder {
    ctx: EncodeContext,
    started: bool,
}

pub struct JsonStructVariantEncoder {
    ctx: EncodeContext,
    started: bool,
}
