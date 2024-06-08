use marshal_core::encode::simple::{SimpleAnyWriter, SimpleWriterAdapter};
use marshal_core::encode::Writer;

use crate::write::{JsonAnyWriter, SimpleJsonWriter};

pub type JsonWriter = SimpleWriterAdapter<SimpleJsonWriter>;

pub struct JsonWriterBuilder {
    inner: SimpleJsonWriter,
}

impl JsonWriterBuilder {
    pub fn new() -> Self {
        JsonWriterBuilder {
            inner: SimpleJsonWriter::new(),
        }
    }
    pub fn build(&mut self) -> <JsonWriter as Writer>::AnyWriter<'_> {
        SimpleAnyWriter::new(&mut self.inner, JsonAnyWriter::new())
    }
    pub fn end(mut self) -> anyhow::Result<String> {
        self.inner.end()
    }
}
