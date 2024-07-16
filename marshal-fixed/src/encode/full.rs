use crate::encode::SimpleFixedSpecEncoder;
use crate::SerializeFixed;
use marshal::context::Context;
use marshal::derive_encoder_for_newtype;
use marshal::encode::poison::PoisonEncoder;
use marshal::encode::{AnySpecEncoder, Encoder};

pub struct FixedSpecEncoder(PoisonEncoder<SimpleFixedSpecEncoder>);

derive_encoder_for_newtype!(FixedSpecEncoder<>(PoisonEncoder<SimpleFixedSpecEncoder>));

pub struct FixedEncoderBuilder {
    inner: FixedSpecEncoder,
}

impl FixedEncoderBuilder {
    pub fn new() -> Self {
        FixedEncoderBuilder {
            inner: FixedSpecEncoder(PoisonEncoder::new(SimpleFixedSpecEncoder::new())),
        }
    }
    pub fn build<'w>(&'w mut self) -> AnySpecEncoder<'w, FixedSpecEncoder> {
        let any = self.inner.0.start(());
        AnySpecEncoder::new(&mut self.inner, any)
    }
    pub fn serialize<T: SerializeFixed>(
        mut self,
        value: &T,
        ctx: Context,
    ) -> anyhow::Result<Vec<u8>> {
        value.serialize(self.build(), ctx)?;
        self.end()
    }
    pub fn end(self) -> anyhow::Result<Vec<u8>> {
        self.inner.0.end()?.end()
    }
}

pub struct FixedEncoder;

impl Encoder for FixedEncoder {
    type SpecEncoder<'en> = FixedSpecEncoder;
}
