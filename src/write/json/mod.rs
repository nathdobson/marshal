mod test;

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

impl Writer for JsonWriter {
    type AnyWriter<'w> = JsonAnyWriter<'w> where Self: 'w;
    type SomeWriter<'w> = JsonSomeWriter<'w> where Self: 'w;
    type TupleWriter<'w> = JsonTupleWriter<'w> where Self: 'w;
    type SeqWriter<'w> = JsonSeqWriter<'w> where Self: 'w;
    type MapWriter<'w> = JsonMapWriter<'w> where Self: 'w;
    type EntryWriter<'w> = JsonEntryWriter<'w> where Self: 'w;
    type TupleStructWriter<'w> = JsonTupleStructWriter<'w> where Self: 'w;
    type StructWriter<'w> = JsonStructWriter<'w>where Self: 'w;
    type TupleVariantWriter<'w> =JsonTupleVariantWriter<'w>where Self: 'w;
    type StructVariantWriter<'w> =JsonStructVariantWriter<'w>where Self: 'w;
}

impl JsonWriter {
    pub fn new() -> Self {
        JsonWriter {
            output: vec![],
            current_indentation: Some(0),
        }
    }
    pub fn start(&mut self) -> JsonAnyWriter<'_> {
        JsonAnyWriter {
            ctx: WriteContext {
                writer: self,
                indentation: 0,
            },
            must_be_string: false,
            cannot_be_null: false,
        }
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
}

struct WriteContext<'w> {
    writer: &'w mut JsonWriter,
    indentation: usize,
}

impl<'w> WriteContext<'w> {
    pub fn write(&mut self, x: fmt::Arguments) -> anyhow::Result<()> {
        self.writer.set_indentation(self.indentation)?;
        write!(&mut self.writer.output, "{}", x)?;
        Ok(())
    }
    pub fn writeln(&mut self, x: fmt::Arguments) -> anyhow::Result<()> {
        self.write(format_args!("{}\n", x))?;
        self.writer.current_indentation = None;
        Ok(())
    }
    pub fn write_null(&mut self) -> anyhow::Result<()> {
        self.write(format_args!("null"))
    }
    pub fn indent_mut(&mut self) -> WriteContext<'_> {
        WriteContext {
            writer: self.writer,
            indentation: self.indentation + 1,
        }
    }
    pub fn indent(self) -> Self {
        WriteContext {
            writer: self.writer,
            indentation: self.indentation + 1,
        }
    }
    pub fn reborrow(&mut self) -> WriteContext<'_> {
        WriteContext {
            writer: self.writer,
            indentation: self.indentation,
        }
    }
    pub fn open(&mut self, x: fmt::Arguments) -> anyhow::Result<()> {
        self.write(x)?;
        Ok(())
    }
    pub fn close(&mut self, x: fmt::Arguments) -> anyhow::Result<()> {
        self.write(x)
    }
    pub fn open_map(&mut self) -> anyhow::Result<()> {
        self.open(format_args!("{{"))
    }
    pub fn close_map(&mut self) -> anyhow::Result<()> {
        self.close(format_args!("}}"))
    }
    pub fn write_str_literal(&mut self, s: &str) -> anyhow::Result<()> {
        self.write(format_args!("\""))?;
        for c in s.chars() {
            match c {
                '\\' => write!(&mut self.writer.output, "\\\\")?,
                '"' => write!(&mut self.writer.output, "\\\"")?,
                '\u{0008}' => write!(&mut self.writer.output, "\\b")?,
                '\u{000c}' => write!(&mut self.writer.output, "\\f")?,
                '\n' => write!(&mut self.writer.output, "\\n")?,
                '\r' => write!(&mut self.writer.output, "\\r")?,
                '\t' => write!(&mut self.writer.output, "\\t")?,
                ..='\u{001f}' => {
                    write!(&mut self.writer.output, "\\u{:0>4x}", c as u32)?;
                }
                _ => write!(&mut self.writer.output, "{}", c)?,
            }
        }
        self.write(format_args!("\""))?;
        Ok(())
    }
    pub fn write_colon(&mut self) -> anyhow::Result<()> {
        self.write(format_args!(": "))
    }
}

pub struct JsonAnyWriter<'w> {
    ctx: WriteContext<'w>,
    must_be_string: bool,
    cannot_be_null: bool,
}

impl<'w> AnyWriter<'w, JsonWriter> for JsonAnyWriter<'w> {
    fn write_prim(mut self, prim: Primitive) -> anyhow::Result<()> {
        match prim {
            Primitive::Unit => self.ctx.write_null(),
            Primitive::Bool(x) => self.ctx.write(format_args!("{}", x)),
            Primitive::I8(x) => self.ctx.write(format_args!("{}", x)),
            Primitive::I16(x) => self.ctx.write(format_args!("{}", x)),
            Primitive::I32(x) => self.ctx.write(format_args!("{}", x)),
            Primitive::I64(x) => self.ctx.write(format_args!("{}", x)),
            Primitive::I128(x) => self.ctx.write(format_args!("{}", x)),
            Primitive::U8(x) => self.ctx.write(format_args!("{}", x)),
            Primitive::U16(x) => self.ctx.write(format_args!("{}", x)),
            Primitive::U32(x) => self.ctx.write(format_args!("{}", x)),
            Primitive::U64(x) => self.ctx.write(format_args!("{}", x)),
            Primitive::U128(x) => self.ctx.write(format_args!("{}", x)),
            Primitive::F32(x) => {
                if x.is_finite() {
                    self.ctx.write(format_args!("{}", x))
                } else {
                    return Err(JsonWriterError::BadNumber)?;
                }
            }
            Primitive::F64(x) => {
                if x.is_finite() {
                    self.ctx.write(format_args!("{}", x))
                } else {
                    return Err(JsonWriterError::BadNumber)?;
                }
            }
            Primitive::Char(x) => self.write_str(x.encode_utf8(&mut [0u8; 4])),
        }
    }

    fn write_str(mut self, s: &str) -> anyhow::Result<()> {
        self.ctx.write_str_literal(s)
    }

    fn write_bytes(self, _: &[u8]) -> anyhow::Result<()> {
        todo!()
    }

    fn write_none(mut self) -> anyhow::Result<()> {
        if self.cannot_be_null {
            let mut map = self.write_map(Some(1))?;
            let mut elem = map.write_entry()?;
            elem.write_key()?.write_str("None")?;
            elem.write_value()?.write_prim(Primitive::Unit)?;
            elem.end()?;
            map.end()?;
        } else {
            self.ctx.write_null()?;
        }
        Ok(())
    }

    fn write_some(self) -> anyhow::Result<<JsonWriter as Writer>::SomeWriter<'w>> {
        Ok(JsonSomeWriter {
            ctx: self.ctx,
            must_be_string: self.must_be_string,
            cannot_be_null: true,
        })
    }

    fn write_unit_struct(mut self, name: &'static str) -> anyhow::Result<()> {
        self.ctx.write_null()
    }

    fn write_tuple_struct(
        self,
        name: &'static str,
        len: usize,
    ) -> anyhow::Result<<JsonWriter as Writer>::TupleStructWriter<'w>> {
        Ok(JsonTupleStructWriter { ctx: self.ctx })
    }

    fn write_struct(
        self,
        name: &'static str,
        len: usize,
    ) -> anyhow::Result<<JsonWriter as Writer>::StructWriter<'w>> {
        Ok(JsonStructWriter { ctx: self.ctx })
    }

    fn write_unit_variant(
        mut self,
        name: &'static str,
        variant_index: u32,
        variant: &'static str,
    ) -> anyhow::Result<()> {
        let mut map = self.write_map(None)?;
        let mut entry = map.write_entry()?;
        entry.write_key()?.write_str(variant)?;
        entry.write_value()?.write_prim(Primitive::Unit)?;
        entry.end()?;
        map.end()?;
        Ok(())
    }

    fn write_tuple_variant(
        self,
        name: &'static str,
        variant_index: u32,
        variant: &'static str,
        len: usize,
    ) -> anyhow::Result<<JsonWriter as Writer>::TupleVariantWriter<'w>> {
        Ok(JsonTupleVariantWriter { ctx: self.ctx })
    }

    fn write_struct_variant(
        self,
        name: &'static str,
        variant_index: u32,
        variant: &'static str,
        len: usize,
    ) -> anyhow::Result<<JsonWriter as Writer>::StructVariantWriter<'w>> {
        Ok(JsonStructVariantWriter { ctx: self.ctx })
    }

    fn write_seq(
        self,
        len: Option<usize>,
    ) -> anyhow::Result<<JsonWriter as Writer>::SeqWriter<'w>> {
        Ok(JsonSeqWriter { ctx: self.ctx })
    }

    fn write_tuple(self, len: usize) -> anyhow::Result<<JsonWriter as Writer>::TupleWriter<'w>> {
        Ok(JsonTupleWriter { ctx: self.ctx })
    }

    fn write_map(
        mut self,
        len: Option<usize>,
    ) -> anyhow::Result<<JsonWriter as Writer>::MapWriter<'w>> {
        self.ctx.open_map()?;
        Ok(JsonMapWriter {
            ctx: self.ctx,
        })
    }
}

pub struct JsonSomeWriter<'w> {
    ctx: WriteContext<'w>,
    must_be_string: bool,
    cannot_be_null: bool,
}

impl<'w> SomeWriter<'w, JsonWriter> for JsonSomeWriter<'w> {
    fn write_some(&mut self) -> anyhow::Result<<JsonWriter as Writer>::AnyWriter<'_>> {
        if self.cannot_be_null {
            self.ctx.open_map()?;
            let mut ctx = self.ctx.indent_mut();
            ctx.write_str_literal("Some")?;
            ctx.write_colon()?;
            Ok(JsonAnyWriter {
                ctx,
                must_be_string: self.must_be_string,
                cannot_be_null: false,
            })
        } else {
            return Ok(JsonAnyWriter {
                ctx: self.ctx.reborrow(),
                must_be_string: self.must_be_string,
                cannot_be_null: true,
            });
        }
    }

    fn end(mut self) -> anyhow::Result<()> {
        if self.cannot_be_null {
            self.ctx.close_map()?;
        }
        Ok(())
    }
}

pub struct JsonTupleWriter<'w> {
    ctx: WriteContext<'w>,
}

impl<'w> TupleWriter<'w, JsonWriter> for JsonTupleWriter<'w> {
    fn write_element(&mut self) -> anyhow::Result<<JsonWriter as Writer>::AnyWriter<'_>> {
        todo!()
    }

    fn end(self) -> anyhow::Result<()> {
        todo!()
    }
}

pub struct JsonSeqWriter<'w> {
    ctx: WriteContext<'w>,
}

impl<'w> SeqWriter<'w, JsonWriter> for JsonSeqWriter<'w> {
    fn write_element(&mut self) -> anyhow::Result<<JsonWriter as Writer>::AnyWriter<'_>> {
        todo!()
    }

    fn end(self) -> anyhow::Result<()> {
        todo!()
    }
}

pub struct JsonMapWriter<'w> {
    ctx: WriteContext<'w>,
}

impl<'w> MapWriter<'w, JsonWriter> for JsonMapWriter<'w> {
    fn write_entry(&mut self) -> anyhow::Result<<JsonWriter as Writer>::EntryWriter<'_>> {
        Ok(JsonEntryWriter {
            ctx: self.ctx.indent_mut(),
        })
    }

    fn end(mut self) -> anyhow::Result<()> {
        self.ctx.close_map()
    }
}
pub struct JsonEntryWriter<'w> {
    ctx: WriteContext<'w>,
}
impl<'w> EntryWriter<'w, JsonWriter> for JsonEntryWriter<'w> {
    fn write_key(&mut self) -> anyhow::Result<<JsonWriter as Writer>::AnyWriter<'_>> {
        Ok(JsonAnyWriter {
            ctx: self.ctx.reborrow(),
            must_be_string: true,
            cannot_be_null: false,
        })
    }

    fn write_value(&mut self) -> anyhow::Result<<JsonWriter as Writer>::AnyWriter<'_>> {
        self.ctx.write_colon()?;
        Ok(JsonAnyWriter {
            ctx: self.ctx.reborrow(),
            must_be_string: false,
            cannot_be_null: false,
        })
    }

    fn end(self) -> anyhow::Result<()> {
        Ok(())
    }
}

pub struct JsonTupleStructWriter<'w> {
    ctx: WriteContext<'w>,
}

impl<'w> TupleStructWriter<'w, JsonWriter> for JsonTupleStructWriter<'w> {
    fn write_field(&mut self) -> anyhow::Result<<JsonWriter as Writer>::AnyWriter<'w>> {
        todo!()
    }

    fn end(self) -> anyhow::Result<()> {
        todo!()
    }
}

pub struct JsonStructWriter<'w> {
    ctx: WriteContext<'w>,
}

impl<'w> StructWriter<'w, JsonWriter> for JsonStructWriter<'w> {
    fn write_field(
        &mut self,
        key: &'static str,
    ) -> anyhow::Result<<JsonWriter as Writer>::AnyWriter<'w>> {
        todo!()
    }

    fn end(self) -> anyhow::Result<()> {
        todo!()
    }
}

pub struct JsonTupleVariantWriter<'w> {
    ctx: WriteContext<'w>,
}

impl<'w> TupleVariantWriter<'w, JsonWriter> for JsonTupleVariantWriter<'w> {
    fn write_field(&mut self) -> anyhow::Result<<JsonWriter as Writer>::AnyWriter<'w>> {
        todo!()
    }

    fn end(self) -> anyhow::Result<()> {
        todo!()
    }
}

pub struct JsonStructVariantWriter<'w> {
    ctx: WriteContext<'w>,
}

impl<'w> StructVariantWriter<'w, JsonWriter> for JsonStructVariantWriter<'w> {
    fn write_field(
        &mut self,
        key: &'static str,
    ) -> anyhow::Result<<JsonWriter as Writer>::AnyWriter<'w>> {
        todo!()
    }

    fn end(self) -> anyhow::Result<()> {
        todo!()
    }
}
