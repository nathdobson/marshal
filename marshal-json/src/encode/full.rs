use marshal::context::Context;
use marshal::ser::Serialize;
use marshal_core::derive_encoder_for_newtype;
use marshal_core::encode::{AnyEncoder, GenEncoder};
use marshal_core::encode::poison::PoisonEncoder;

use crate::encode::{JsonAnyEncoder, SimpleJsonSpecEncoder};

pub struct JsonSpecEncoder(PoisonEncoder<SimpleJsonSpecEncoder>);

derive_encoder_for_newtype!(JsonSpecEncoder(PoisonEncoder<SimpleJsonSpecEncoder>));

pub struct JsonEncoderBuilder {
    inner: JsonSpecEncoder,
}

impl JsonEncoderBuilder {
    pub fn new() -> Self {
        JsonEncoderBuilder {
            inner: JsonSpecEncoder(PoisonEncoder::new(SimpleJsonSpecEncoder::new())),
        }
    }
    pub fn build(&mut self) -> AnyEncoder<'_, JsonSpecEncoder> {
        let any = self.inner.0.start(JsonAnyEncoder::new());
        AnyEncoder::new(&mut self.inner, any)
    }
    pub fn end(mut self) -> anyhow::Result<String> {
        Ok(self.inner.0.end()?.end()?)
    }
    pub fn with<F: FnOnce(AnyEncoder<JsonSpecEncoder>) -> anyhow::Result<()>>(
        mut self,
        f: F,
    ) -> anyhow::Result<String> {
        f(self.build())?;
        self.end()
    }
    pub fn serialize<T: ?Sized + Serialize<JsonEncoder>>(
        mut self,
        value: &T,
        mut ctx: Context,
    ) -> anyhow::Result<String> {
        value.serialize(self.build(), ctx)?;
        self.end()
    }
}

pub struct JsonEncoder;

impl GenEncoder for JsonEncoder {
    type SpecEncoder<'en> = JsonSpecEncoder;
}
