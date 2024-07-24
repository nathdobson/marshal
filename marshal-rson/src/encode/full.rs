use crate::encode::{RsonAnySpecEncoder, SimpleRsonSpecEncoder};
use marshal::context::Context;
use marshal::derive_encoder_for_newtype;
use marshal::encode::poison::PoisonEncoder;
use marshal::encode::{AnySpecEncoder, Encoder};
use marshal::reexports::anyhow;
use marshal::ser::Serialize;

pub struct RsonSpecEncoder(PoisonEncoder<SimpleRsonSpecEncoder>);

derive_encoder_for_newtype!(RsonSpecEncoder(PoisonEncoder<SimpleRsonSpecEncoder>));

pub struct RsonEncoderBuilder {
    inner: RsonSpecEncoder,
}

impl RsonEncoderBuilder {
    pub fn new() -> Self {
        RsonEncoderBuilder {
            inner: RsonSpecEncoder(PoisonEncoder::new(SimpleRsonSpecEncoder::new())),
        }
    }
    pub fn build(&mut self) -> AnySpecEncoder<'_, RsonSpecEncoder> {
        let any = self.inner.0.start(RsonAnySpecEncoder::new());
        AnySpecEncoder::new(&mut self.inner, any)
    }
    pub fn end(self) -> anyhow::Result<String> {
        Ok(self.inner.0.end()?.end()?)
    }
    pub fn with<F: FnOnce(AnySpecEncoder<RsonSpecEncoder>) -> anyhow::Result<()>>(
        mut self,
        f: F,
    ) -> anyhow::Result<String> {
        f(self.build())?;
        self.end()
    }
    pub fn serialize<T: ?Sized + Serialize<RsonEncoder>>(
        mut self,
        value: &T,
        ctx: Context,
    ) -> anyhow::Result<String> {
        value.serialize(self.build(), ctx)?;
        self.end()
    }
}

pub struct RsonEncoder;

impl Encoder for RsonEncoder {
    type SpecEncoder<'en> = RsonSpecEncoder;
}
