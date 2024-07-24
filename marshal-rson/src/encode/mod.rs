use crate::RsonError;
use anyhow;
use base64::prelude::BASE64_STANDARD;
use base64::Engine;
use marshal::encode::SpecEncoder;
use marshal::Primitive;
use std::fmt::Display;
use std::fmt::Write;

pub mod full;

pub struct SimpleRsonSpecEncoder {
    indentation: Option<usize>,
    output: String,
}

impl SimpleRsonSpecEncoder {
    pub fn new() -> Self {
        SimpleRsonSpecEncoder {
            indentation: None,
            output: String::new(),
        }
    }
    fn set_indentation(&mut self, ctx: EncodeContext) {
        if let Some(current) = self.indentation {
            if current != ctx.indentation {
                self.output.push_str("\n");
                self.indentation = None;
            }
        }
        if self.indentation.is_none() {
            for _ in 0..ctx.indentation * 2 {
                self.output.push(' ');
            }
            self.indentation = Some(ctx.indentation);
        }
    }
    fn append_str(&mut self, ctx: EncodeContext, s: &str) {
        self.set_indentation(ctx);
        self.output.push_str(s);
    }
    fn append_line(&mut self) {
        self.indentation = None;
        self.output.push('\n');
    }
    fn append_display(&mut self, ctx: EncodeContext, s: impl Display) {
        self.set_indentation(ctx);
        write!(&mut self.output, "{}", s).unwrap();
    }
    pub fn end(self) -> anyhow::Result<String> {
        Ok(self.output)
    }
    fn append_delimiter(&mut self, ctx: &mut BlockContext, d: &str, s: &str) -> EncodeContext {
        if ctx.len > 1 {
            if ctx.started {
                self.append_str(ctx.encode_context.indent(), d);
                self.append_line();
            }
            ctx.started = true;
            ctx.encode_context.indent()
        } else {
            self.append_str(ctx.encode_context, s);
            ctx.encode_context
        }
    }
    fn append_terminator(&mut self, ctx: BlockContext, d: &str, s: &str, t: &str) {
        if ctx.started {
            self.append_str(ctx.encode_context.indent(), d);
        }
        if ctx.len == 1 {
            self.append_str(ctx.encode_context, s);
        }
        self.append_str(ctx.encode_context, t);
    }
}

#[derive(Copy, Clone)]
struct EncodeContext {
    indentation: usize,
}

struct BlockContext {
    encode_context: EncodeContext,
    started: bool,
    len: usize,
}

pub struct RsonAnySpecEncoder {
    ctx: EncodeContext,
}

impl RsonAnySpecEncoder {
    pub fn new() -> Self {
        RsonAnySpecEncoder {
            ctx: EncodeContext::new(),
        }
    }
}

impl EncodeContext {
    pub fn new() -> Self {
        EncodeContext { indentation: 0 }
    }
}

pub struct RsonTupleStructEncoder {
    ctx: BlockContext,
}

pub struct RsonStructEncoder {
    ctx: BlockContext,
}

pub struct RsonTupleVariantEncoder {
    ctx: BlockContext,
}

pub struct RsonStructVariantEncoder {
    ctx: BlockContext,
}

pub struct RsonTupleEncoder {
    ctx: BlockContext,
}

pub struct RsonSeqEncoder {
    ctx: BlockContext,
}

pub struct RsonMapEncoder {
    ctx: BlockContext,
}

pub struct RsonValueEncoder {
    ctx: EncodeContext,
}

impl EncodeContext {
    pub fn indent(self) -> Self {
        EncodeContext {
            indentation: self.indentation + 1,
        }
    }
}

impl BlockContext {
    pub fn new(ctx: EncodeContext, len: usize) -> Self {
        BlockContext {
            encode_context: ctx,
            started: false,
            len,
        }
    }
}

impl SpecEncoder for SimpleRsonSpecEncoder {
    type AnySpecEncoder = RsonAnySpecEncoder;
    type SomeCloser = ();
    type TupleEncoder = RsonTupleEncoder;
    type SeqEncoder = RsonSeqEncoder;
    type MapEncoder = RsonMapEncoder;
    type ValueEncoder = RsonValueEncoder;
    type EntryCloser = ();
    type TupleStructEncoder = RsonTupleStructEncoder;
    type StructEncoder = RsonStructEncoder;
    type TupleVariantEncoder = RsonTupleVariantEncoder;
    type StructVariantEncoder = RsonStructVariantEncoder;

    fn encode_prim(&mut self, any: Self::AnySpecEncoder, prim: Primitive) -> anyhow::Result<()> {
        let ctx = any.ctx;
        match prim {
            Primitive::Unit => self.append_str(ctx, "unit"),
            Primitive::Bool(x) => self.append_display(ctx, format_args!("{}", x)),
            Primitive::I8(x) => self.append_display(ctx, format_args!("i8 {}", x)),
            Primitive::I16(x) => self.append_display(ctx, format_args!("i16 {}", x)),
            Primitive::I32(x) => self.append_display(ctx, format_args!("i32 {}", x)),
            Primitive::I64(x) => self.append_display(ctx, format_args!("i64 {}", x)),
            Primitive::I128(x) => self.append_display(ctx, format_args!("i128 {}", x)),
            Primitive::U8(x) => self.append_display(ctx, format_args!("u8 {}", x)),
            Primitive::U16(x) => self.append_display(ctx, format_args!("u16 {}", x)),
            Primitive::U32(x) => self.append_display(ctx, format_args!("u32 {}", x)),
            Primitive::U64(x) => self.append_display(ctx, format_args!("u64 {}", x)),
            Primitive::U128(x) => self.append_display(ctx, format_args!("u128 {}", x)),
            Primitive::F32(x) => self.append_display(ctx, format_args!("f32 {}", x)),
            Primitive::F64(x) => self.append_display(ctx, format_args!("f64 {}", x)),
            Primitive::Char(x) => self.append_display(ctx, format_args!("char '{}'", x)),
        }
        Ok(())
    }

    fn encode_str(&mut self, any: Self::AnySpecEncoder, s: &str) -> anyhow::Result<()> {
        self.set_indentation(any.ctx);
        self.output.push_str("string \"");
        for c in s.chars() {
            if c == '\"' {
                self.output.push_str("\\\"");
            } else if c == '\n' {
                self.output.push_str("\\n");
            } else if c == '\\' {
                self.output.push_str("\\\\");
            } else {
                self.output.push(c);
            }
        }
        self.output.push('"');
        Ok(())
    }

    fn encode_bytes(&mut self, any: Self::AnySpecEncoder, s: &[u8]) -> anyhow::Result<()> {
        self.set_indentation(any.ctx);
        self.output.push_str("bytes \"");
        BASE64_STANDARD.encode_string(s, &mut self.output);
        self.output.push('"');
        Ok(())
    }

    fn encode_none(&mut self, any: Self::AnySpecEncoder) -> anyhow::Result<()> {
        self.append_str(any.ctx, "none");
        Ok(())
    }

    fn encode_some(
        &mut self,
        any: Self::AnySpecEncoder,
    ) -> anyhow::Result<(Self::AnySpecEncoder, Self::SomeCloser)> {
        self.append_str(any.ctx, "some ");
        Ok((any, ()))
    }

    fn encode_unit_struct(
        &mut self,
        any: Self::AnySpecEncoder,
        name: &'static str,
    ) -> anyhow::Result<()> {
        self.append_display(any.ctx, format_args!("struct {}", name));
        Ok(())
    }

    fn encode_tuple_struct(
        &mut self,
        any: Self::AnySpecEncoder,
        name: &'static str,
        len: usize,
    ) -> anyhow::Result<Self::TupleStructEncoder> {
        self.append_display(any.ctx, format_args!("struct {}(", name));
        Ok(RsonTupleStructEncoder {
            ctx: BlockContext::new(any.ctx, len),
        })
    }

    fn encode_struct(
        &mut self,
        any: Self::AnySpecEncoder,
        name: &'static str,
        fields: &'static [&'static str],
    ) -> anyhow::Result<Self::StructEncoder> {
        self.append_display(any.ctx, format_args!("struct {} {{", name));
        Ok(RsonStructEncoder {
            ctx: BlockContext::new(any.ctx, fields.len()),
        })
    }

    fn encode_unit_variant(
        &mut self,
        any: Self::AnySpecEncoder,
        name: &'static str,
        variants: &'static [&'static str],
        variant_index: usize,
    ) -> anyhow::Result<()> {
        self.append_display(
            any.ctx,
            format_args!("enum {}::{}", name, variants[variant_index]),
        );
        Ok(())
    }

    fn encode_tuple_variant(
        &mut self,
        any: Self::AnySpecEncoder,
        name: &'static str,
        variants: &'static [&'static str],
        variant_index: usize,
        len: usize,
    ) -> anyhow::Result<Self::TupleVariantEncoder> {
        self.append_display(
            any.ctx,
            format_args!("enum {}::{}(", name, variants[variant_index]),
        );
        Ok(RsonTupleVariantEncoder {
            ctx: BlockContext::new(any.ctx, len),
        })
    }

    fn encode_struct_variant(
        &mut self,
        any: Self::AnySpecEncoder,
        name: &'static str,
        variants: &'static [&'static str],
        variant_index: usize,
        fields: &'static [&'static str],
    ) -> anyhow::Result<Self::StructVariantEncoder> {
        self.append_display(
            any.ctx,
            format_args!("enum {}::{} {{", name, variants[variant_index]),
        );
        Ok(RsonStructVariantEncoder {
            ctx: BlockContext::new(any.ctx, fields.len()),
        })
    }

    fn encode_seq(
        &mut self,
        any: Self::AnySpecEncoder,
        len: Option<usize>,
    ) -> anyhow::Result<Self::SeqEncoder> {
        let len = len.ok_or(RsonError::MissingLen)?;
        self.append_str(any.ctx, "[");
        Ok(RsonSeqEncoder {
            ctx: BlockContext::new(any.ctx, len),
        })
    }

    fn encode_tuple(
        &mut self,
        any: Self::AnySpecEncoder,
        len: usize,
    ) -> anyhow::Result<Self::TupleEncoder> {
        self.append_str(any.ctx, "(");
        Ok(RsonTupleEncoder {
            ctx: BlockContext::new(any.ctx, len),
        })
    }

    fn encode_map(
        &mut self,
        any: Self::AnySpecEncoder,
        len: Option<usize>,
    ) -> anyhow::Result<Self::MapEncoder> {
        let len = len.ok_or(RsonError::MissingLen)?;
        self.append_str(any.ctx, "{");
        Ok(RsonMapEncoder {
            ctx: BlockContext::new(any.ctx, len),
        })
    }

    fn some_end(&mut self, _some: Self::SomeCloser) -> anyhow::Result<()> {
        Ok(())
    }

    fn tuple_encode_element(
        &mut self,
        tuple: &mut Self::TupleEncoder,
    ) -> anyhow::Result<Self::AnySpecEncoder> {
        Ok(RsonAnySpecEncoder {
            ctx: self.append_delimiter(&mut tuple.ctx, ",", ""),
        })
    }

    fn tuple_end(&mut self, tuple: Self::TupleEncoder) -> anyhow::Result<()> {
        self.append_terminator(tuple.ctx, ",", ",", ")");
        Ok(())
    }

    fn seq_encode_element(
        &mut self,
        seq: &mut Self::SeqEncoder,
    ) -> anyhow::Result<Self::AnySpecEncoder> {
        Ok(RsonAnySpecEncoder {
            ctx: self.append_delimiter(&mut seq.ctx, ",", ""),
        })
    }

    fn seq_end(&mut self, tuple: Self::SeqEncoder) -> anyhow::Result<()> {
        self.append_terminator(tuple.ctx, ",", "", "]");
        Ok(())
    }

    fn map_encode_element(
        &mut self,
        map: &mut Self::MapEncoder,
    ) -> anyhow::Result<(Self::AnySpecEncoder, Self::ValueEncoder)> {
        let ctx = self.append_delimiter(&mut map.ctx, ",", " ");
        Ok((RsonAnySpecEncoder { ctx }, RsonValueEncoder { ctx }))
    }

    fn map_end(&mut self, map: Self::MapEncoder) -> anyhow::Result<()> {
        self.append_terminator(map.ctx, ",", " ", "}");
        Ok(())
    }

    fn entry_encode_value(
        &mut self,
        value: Self::ValueEncoder,
    ) -> anyhow::Result<(Self::AnySpecEncoder, Self::EntryCloser)> {
        self.append_str(value.ctx, ": ");
        Ok((RsonAnySpecEncoder { ctx: value.ctx }, ()))
    }

    fn entry_end(&mut self, _closer: Self::EntryCloser) -> anyhow::Result<()> {
        Ok(())
    }

    fn tuple_struct_encode_field(
        &mut self,
        map: &mut Self::TupleStructEncoder,
    ) -> anyhow::Result<Self::AnySpecEncoder> {
        Ok(RsonAnySpecEncoder {
            ctx: self.append_delimiter(&mut map.ctx, ",", ""),
        })
    }

    fn tuple_struct_end(&mut self, map: Self::TupleStructEncoder) -> anyhow::Result<()> {
        self.append_terminator(map.ctx, ",", "", ")");
        Ok(())
    }

    fn struct_encode_field(
        &mut self,
        map: &mut Self::StructEncoder,
        field: &'static str,
    ) -> anyhow::Result<Self::AnySpecEncoder> {
        let ctx = self.append_delimiter(&mut map.ctx, ",", " ");
        self.append_display(ctx, format_args!("{}: ", field));
        Ok(RsonAnySpecEncoder { ctx })
    }

    fn struct_end(&mut self, map: Self::StructEncoder) -> anyhow::Result<()> {
        self.append_terminator(map.ctx, ",", " ", "}");
        Ok(())
    }

    fn tuple_variant_encode_field(
        &mut self,
        map: &mut Self::TupleVariantEncoder,
    ) -> anyhow::Result<Self::AnySpecEncoder> {
        let ctx = self.append_delimiter(&mut map.ctx, ",", "");
        Ok(RsonAnySpecEncoder { ctx })
    }

    fn tuple_variant_end(&mut self, map: Self::TupleVariantEncoder) -> anyhow::Result<()> {
        self.append_terminator(map.ctx, ",", "", ")");
        Ok(())
    }

    fn struct_variant_encode_field(
        &mut self,
        map: &mut Self::StructVariantEncoder,
        key: &'static str,
    ) -> anyhow::Result<Self::AnySpecEncoder> {
        let ctx = self.append_delimiter(&mut map.ctx, ",", " ");
        self.append_display(ctx, format_args!("{}: ", key));
        Ok(RsonAnySpecEncoder { ctx })
    }

    fn struct_variant_end(&mut self, map: Self::StructVariantEncoder) -> anyhow::Result<()> {
        self.append_terminator(map.ctx, ",", " ", "}");
        Ok(())
    }

    fn is_human_readable(&self) -> bool {
        true
    }
}
