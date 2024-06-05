mod test;

use crate::write::simple::{SimpleAnyWriter, SimpleWriter};
use crate::write::{
    AnyWriter, EntryWriter, MapWriter, SeqWriter, SomeWriter, StructVariantWriter, StructWriter,
    TupleStructWriter, TupleVariantWriter, TupleWriter, Writer,
};
use crate::Primitive;
use std::error::Error;
use std::fmt;
use std::fmt::{Display, Formatter};
use std::io::Write;

pub struct JsonWriter {
    output: Vec<u8>,
    current_indentation: Option<usize>,
}

#[derive(Debug)]
pub enum JsonWriterError {
    BadNumber,
}

impl Display for JsonWriterError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "JsonWriterError")
    }
}

impl Error for JsonWriterError {}

impl JsonWriter {
    pub fn new() -> Self {
        JsonWriter {
            output: vec![],
            current_indentation: Some(0),
        }
    }
    pub fn start(&mut self) -> SimpleAnyWriter<JsonWriter> {
        SimpleAnyWriter::new(
            self,
            JsonAnyWriter {
                ctx: WriteContext { indentation: 0 },
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

    fn write(&mut self, ctx: WriteContext, value: impl Display) -> anyhow::Result<()> {
        self.set_indentation(ctx.indentation)?;
        write!(&mut self.output, "{}", value)?;
        Ok(())
    }
    fn writeln(&mut self, ctx: WriteContext, value: impl Display) -> anyhow::Result<()> {
        self.set_indentation(ctx.indentation)?;
        write!(&mut self.output, "{}\n", value)?;
        self.current_indentation = None;
        Ok(())
    }
    pub fn write_null(&mut self, ctx: WriteContext) -> anyhow::Result<()> {
        self.write(ctx, "null")
    }
    pub fn open_map(&mut self, ctx: WriteContext) -> anyhow::Result<()> {
        self.write(ctx, "{")
    }
    pub fn close_map(&mut self, ctx: WriteContext) -> anyhow::Result<()> {
        self.write(ctx, "}")
    }
    pub fn open_list(&mut self, ctx: WriteContext) -> anyhow::Result<()> {
        self.write(ctx, "[")
    }
    pub fn close_list(&mut self, ctx: WriteContext) -> anyhow::Result<()> {
        self.write(ctx, "]")
    }
    pub fn write_str_literal(&mut self, ctx: WriteContext, s: &str) -> anyhow::Result<()> {
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
    pub fn write_colon(&mut self, ctx: WriteContext) -> anyhow::Result<()> {
        self.write(ctx, ": ")
    }
    pub fn write_comma(&mut self, ctx: WriteContext) -> anyhow::Result<()> {
        self.writeln(ctx, ",")
    }
}

impl SimpleWriter for JsonWriter {
    type AnyWriter = JsonAnyWriter;
    type SomeCloser = JsonSomeCloser;
    type TupleWriter = JsonTupleWriter;
    type SeqWriter = JsonSeqWriter;
    type MapWriter = JsonMapWriter;
    type ValueWriter = JsonValueWriter;
    type EntryCloser = JsonEntryCloser;
    type TupleStructWriter = JsonTupleStructWriter;
    type StructWriter = JsonStructWriter;
    type TupleVariantWriter = JsonTupleVariantWriter;
    type StructVariantWriter = JsonStructVariantWriter;

    fn write_prim(&mut self, any: Self::AnyWriter, prim: Primitive) -> anyhow::Result<()> {
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
                    return Err(JsonWriterError::BadNumber)?;
                }
            }
            Primitive::F64(x) => {
                if x.is_finite() {
                    self.write(any.ctx, x)
                } else {
                    return Err(JsonWriterError::BadNumber)?;
                }
            }
            Primitive::Char(x) => self.write_str_literal(any.ctx, x.encode_utf8(&mut [0u8; 4])),
        }
    }

    fn write_str(&mut self, any: Self::AnyWriter, s: &str) -> anyhow::Result<()> {
        self.write_str_literal(any.ctx, s)
    }

    fn write_bytes(&mut self, any: Self::AnyWriter, s: &[u8]) -> anyhow::Result<()> {
        todo!()
    }

    fn write_none(&mut self, any: Self::AnyWriter) -> anyhow::Result<()> {
        todo!()
    }

    fn write_some(
        &mut self,
        any: Self::AnyWriter,
    ) -> anyhow::Result<(Self::AnyWriter, Self::SomeCloser)> {
        todo!()
    }

    fn write_unit_struct(
        &mut self,
        any: Self::AnyWriter,
        name: &'static str,
    ) -> anyhow::Result<()> {
        todo!()
    }

    fn write_tuple_struct(
        &mut self,
        any: Self::AnyWriter,
        name: &'static str,
        len: usize,
    ) -> anyhow::Result<Self::TupleStructWriter> {
        todo!()
    }

    fn write_struct(
        &mut self,
        any: Self::AnyWriter,
        name: &'static str,
        len: usize,
    ) -> anyhow::Result<Self::StructWriter> {
        todo!()
    }

    fn write_unit_variant(
        &mut self,
        any: Self::AnyWriter,
        name: &'static str,
        variant_index: u32,
        variant: &'static str,
    ) -> anyhow::Result<()> {
        todo!()
    }

    fn write_tuple_variant(
        &mut self,
        any: Self::AnyWriter,
        name: &'static str,
        variant_index: u32,
        variant: &'static str,
        len: usize,
    ) -> anyhow::Result<Self::TupleVariantWriter> {
        todo!()
    }

    fn write_struct_variant(
        &mut self,
        any: Self::AnyWriter,
        name: &'static str,
        variant_index: u32,
        variant: &'static str,
        len: usize,
    ) -> anyhow::Result<Self::StructVariantWriter> {
        todo!()
    }

    fn write_seq(
        &mut self,
        any: Self::AnyWriter,
        len: Option<usize>,
    ) -> anyhow::Result<Self::SeqWriter> {
        self.open_list(any.ctx)?;
        Ok(JsonSeqWriter {
            ctx: any.ctx,
            started: false,
        })
    }

    fn write_tuple(
        &mut self,
        any: Self::AnyWriter,
        len: usize,
    ) -> anyhow::Result<Self::TupleWriter> {
        todo!()
    }

    fn write_map(
        &mut self,
        any: Self::AnyWriter,
        len: Option<usize>,
    ) -> anyhow::Result<Self::MapWriter> {
        self.open_map(any.ctx)?;
        Ok(JsonMapWriter {
            ctx: any.ctx,
            started: false,
        })
    }

    fn some_end(&mut self, some: Self::SomeCloser) -> anyhow::Result<()> {
        todo!()
    }

    fn tuple_write_element(
        &mut self,
        tuple: &mut Self::TupleWriter,
    ) -> anyhow::Result<Self::AnyWriter> {
        todo!()
    }

    fn tuple_end(&mut self, tuple: Self::TupleWriter) -> anyhow::Result<()> {
        todo!()
    }

    fn seq_write_element(&mut self, seq: &mut Self::SeqWriter) -> anyhow::Result<Self::AnyWriter> {
        let ctx = seq.ctx.indent();
        if seq.started {
            self.write_comma(ctx)?;
        }
        seq.started = true;
        Ok(JsonAnyWriter {
            ctx,
            must_be_string: false,
            cannot_be_null: false,
        })
    }

    fn seq_end(&mut self, tuple: Self::SeqWriter) -> anyhow::Result<()> {
        self.close_list(tuple.ctx)
    }

    fn map_write_element(
        &mut self,
        map: &mut Self::MapWriter,
    ) -> anyhow::Result<(Self::AnyWriter, Self::ValueWriter)> {
        let ctx = map.ctx.indent();
        if map.started {
            self.write_comma(ctx)?;
        }
        map.started = true;
        Ok((
            JsonAnyWriter {
                ctx,
                must_be_string: true,
                cannot_be_null: false,
            },
            JsonValueWriter { ctx },
        ))
    }

    fn map_end(&mut self, map: Self::MapWriter) -> anyhow::Result<()> {
        self.close_map(map.ctx)?;
        Ok(())
    }

    fn entry_write_value(
        &mut self,
        value: Self::ValueWriter,
    ) -> anyhow::Result<(Self::AnyWriter, Self::EntryCloser)> {
        self.write_colon(value.ctx)?;
        Ok((
            JsonAnyWriter {
                ctx: value.ctx,
                must_be_string: false,
                cannot_be_null: false,
            },
            JsonEntryCloser { ctx: value.ctx },
        ))
    }

    fn entry_end(&mut self, closer: Self::EntryCloser) -> anyhow::Result<()> {
        Ok(())
    }

    fn tuple_struct_write_field(
        &mut self,
        map: &mut Self::TupleStructWriter,
    ) -> anyhow::Result<Self::AnyWriter> {
        todo!()
    }

    fn tuple_struct_end(&mut self, map: Self::TupleStructWriter) -> anyhow::Result<()> {
        todo!()
    }

    fn struct_write_field(
        &mut self,
        map: &mut Self::StructWriter,
        key: &'static str,
    ) -> anyhow::Result<Self::AnyWriter> {
        todo!()
    }

    fn struct_end(&mut self, map: Self::StructWriter) -> anyhow::Result<()> {
        todo!()
    }

    fn tuple_variant_write_field(
        &mut self,
        map: &mut Self::TupleVariantWriter,
    ) -> anyhow::Result<Self::AnyWriter> {
        todo!()
    }

    fn tuple_variant_end(&mut self, map: Self::TupleVariantWriter) -> anyhow::Result<()> {
        todo!()
    }

    fn struct_variant_write_field(
        &mut self,
        map: &mut Self::StructVariantWriter,
        key: &'static str,
    ) -> anyhow::Result<Self::AnyWriter> {
        todo!()
    }

    fn struct_variant_end(&mut self, map: Self::StructVariantWriter) -> anyhow::Result<()> {
        todo!()
    }
}

#[derive(Copy, Clone)]
struct WriteContext {
    indentation: usize,
}

impl WriteContext {
    pub fn indent(self) -> Self {
        WriteContext {
            indentation: self.indentation + 1,
        }
    }
}

pub struct JsonAnyWriter {
    ctx: WriteContext,
    must_be_string: bool,
    cannot_be_null: bool,
}

pub struct JsonSomeWriter {
    ctx: WriteContext,
    must_be_string: bool,
    cannot_be_null: bool,
}

pub struct JsonSomeCloser {
    ctx: WriteContext,
    cannot_be_null: bool,
}

pub struct JsonTupleWriter {
    ctx: WriteContext,
}

pub struct JsonSeqWriter {
    ctx: WriteContext,
    started: bool,
}

pub struct JsonMapWriter {
    ctx: WriteContext,
    started: bool,
}

pub struct JsonKeyWriter {
    ctx: WriteContext,
}

pub struct JsonValueWriter {
    ctx: WriteContext,
}

pub struct JsonEntryCloser {
    ctx: WriteContext,
}

pub struct JsonTupleStructWriter {
    ctx: WriteContext,
}

pub struct JsonStructWriter {
    ctx: WriteContext,
}

pub struct JsonTupleVariantWriter {
    ctx: WriteContext,
}

pub struct JsonStructVariantWriter {
    ctx: WriteContext,
}
