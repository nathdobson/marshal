use marshal::context::Context;
use marshal::ser::Serialize;
use marshal_core::derive_encoder_for_newtype;
use marshal_core::encode::poison::PoisonEncoder;
use marshal_core::encode::{AnyEncoder, GenEncoder};

use crate::encode::{JsonAnyEncoder, SimpleJsonEncoder};

pub struct JsonEncoder(PoisonEncoder<SimpleJsonEncoder>);

derive_encoder_for_newtype!(JsonEncoder(PoisonEncoder<SimpleJsonEncoder>));

pub struct JsonEncoderBuilder {
    inner: JsonEncoder,
}

impl JsonEncoderBuilder {
    pub fn new() -> Self {
        JsonEncoderBuilder {
            inner: JsonEncoder(PoisonEncoder::new(SimpleJsonEncoder::new())),
        }
    }
    pub fn build(&mut self) -> AnyEncoder<'_, JsonEncoder> {
        let any = self.inner.0.start(JsonAnyEncoder::new());
        AnyEncoder::new(&mut self.inner, any)
    }
    pub fn end(mut self) -> anyhow::Result<String> {
        Ok(self.inner.0.end()?.end()?)
    }
    pub fn with<F: FnOnce(AnyEncoder<JsonEncoder>) -> anyhow::Result<()>>(
        mut self,
        f: F,
    ) -> anyhow::Result<String> {
        f(self.build())?;
        self.end()
    }
    pub fn serialize<T: ?Sized + Serialize<JsonGenEncoder>>(
        mut self,
        value: &T,
        mut ctx: Context,
    ) -> anyhow::Result<String> {
        value.serialize(self.build(), ctx)?;
        self.end()
    }
}

pub struct JsonGenEncoder;

impl GenEncoder for JsonGenEncoder {
    type Encoder<'en> = JsonEncoder;
}
