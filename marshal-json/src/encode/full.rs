use marshal_core::encode::simple::{SimpleAnyEncoder, SimpleEncoderAdapter};
use marshal_core::encode::Encoder;

use crate::encode::{JsonAnyEncoder, SimpleJsonEncoder};

pub type JsonEncoder = SimpleEncoderAdapter<SimpleJsonEncoder>;

pub struct JsonEncoderBuilder {
    inner: SimpleJsonEncoder,
}

impl JsonEncoderBuilder {
    pub fn new() -> Self {
        JsonEncoderBuilder {
            inner: SimpleJsonEncoder::new(),
        }
    }
    pub fn build(&mut self) -> <JsonEncoder as Encoder>::AnyEncoder<'_> {
        SimpleAnyEncoder::new(&mut self.inner, JsonAnyEncoder::new())
    }
    pub fn end(mut self) -> anyhow::Result<String> {
        self.inner.end()
    }
}
