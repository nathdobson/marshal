use crate::write::{BinWriterSchema, SimpleBinWriter};
use marshal_core::write::simple::{SimpleAnyWriter, SimpleWriterAdapter};
use marshal_core::write::Writer;

pub type BinWriter<'s> = SimpleWriterAdapter<SimpleBinWriter<'s>>;

pub struct BinWriterBuilder<'s> {
    inner: SimpleBinWriter<'s>,
}

impl<'s> BinWriterBuilder<'s> {
    pub fn new(schema: &'s mut BinWriterSchema) -> Self {
        BinWriterBuilder {
            inner: SimpleBinWriter::new(schema),
        }
    }
    pub fn build<'w>(&'w mut self) -> <BinWriter<'s> as Writer>::AnyWriter<'w> {
        SimpleAnyWriter::new(&mut self.inner, ())
    }
    pub fn end(self) -> anyhow::Result<Vec<u8>> {
        self.inner.end()
    }
}
