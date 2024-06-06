use crate::write::SimpleJsonWriter;
use marshal_core::write::simple::{SimpleAnyWriter, SimpleWriter, SimpleWriterAdapter};

pub type JsonWriter = SimpleWriterAdapter<SimpleJsonWriter>;
