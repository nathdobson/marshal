use marshal::context::Context;
use marshal::ser::Serialize;
use marshal_core::encode::AnyEncoder;
use marshal_core::encode::poison::PoisonEncoder;

use crate::encode::{JsonAnyEncoder, SimpleJsonEncoder};

pub type JsonEncoder = PoisonEncoder<SimpleJsonEncoder>;

pub struct JsonEncoderBuilder {
    inner: PoisonEncoder<SimpleJsonEncoder>,
}

impl JsonEncoderBuilder {
    pub fn new() -> Self {
        JsonEncoderBuilder {
            inner: PoisonEncoder::new(SimpleJsonEncoder::new()),
        }
    }
    pub fn build(&mut self) -> AnyEncoder<'_, JsonEncoder> {
        let any = self.inner.start(JsonAnyEncoder::new());
        AnyEncoder::new(&mut self.inner, any)
    }
    pub fn end(mut self) -> anyhow::Result<String> {
        Ok(self.inner.end()?.end()?)
    }
    pub fn serialize<T: Serialize<JsonEncoder>>(
        mut self,
        value: &T,
        ctx: &mut Context,
    ) -> anyhow::Result<String> {
        value.serialize(self.build(), ctx)?;
        self.end()
    }
}
