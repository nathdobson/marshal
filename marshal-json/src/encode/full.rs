use marshal::context::Context;
use marshal::ser::Serialize;
use marshal_core::encode::{AnyEncoder, Encoder};

use crate::encode::{JsonAnyEncoder, SimpleJsonEncoder};

pub type JsonEncoder = SimpleJsonEncoder;

pub struct JsonEncoderBuilder {
    // poison: PoisonState,
    inner: SimpleJsonEncoder,
}

impl JsonEncoderBuilder {
    pub fn new() -> Self {
        JsonEncoderBuilder {
            // poison: PoisonState::new(),
            inner: SimpleJsonEncoder::new(),
        }
    }
    pub fn build(&mut self) -> AnyEncoder<'_, JsonEncoder> {
        AnyEncoder::new(&mut self.inner, JsonAnyEncoder::new())
    }
    pub fn end(mut self) -> anyhow::Result<String> {
        // self.poison.check()?;
        self.inner.end()
    }
    pub fn serialize<T: Serialize<JsonEncoder>>(
        mut self,
        value: &T,
        ctx: &mut Context,
    ) -> anyhow::Result<String> {
        value.serialize(self.build(), ctx)?;
        // self.poison.check()?;
        self.inner.end()
    }
}
