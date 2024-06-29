use std::any::Any;
use std::sync;

use pretty_assertions::assert_eq;

use marshal::context::OwnedContext;
use marshal_json::decode::full::{JsonDecoderBuilder, JsonDecoder};
use marshal_json::encode::full::{JsonEncoderBuilder, JsonEncoder};
use marshal_shared::de::SharedArcDeserializeContext;
use marshal_shared::ser::SharedSerializeContext;

use crate::de::DeserializeUpdate;
use crate::forest::forest::Tree;
use crate::ser::{SerializeStream, SerializeUpdate};

pub struct Tester<T: SerializeStream> {
    shared_ser_ctx: SharedSerializeContext<sync::Weak<Tree<dyn Sync + Send + Any>>>,
    input: T,
    input_stream: T::Stream,
    shared_de_ctx: SharedArcDeserializeContext,
    output: T,
}

impl<
        T: SerializeStream
            + SerializeUpdate<JsonEncoder>
            + DeserializeUpdate<JsonDecoder>,
    > Tester<T>
{
    #[track_caller]
    pub fn new(input: T, expected: &str) -> anyhow::Result<Self> {
        let mut shared_ser_ctx = SharedSerializeContext::default();
        let mut encode_ctx = OwnedContext::new();
        encode_ctx.insert_mut(&mut shared_ser_ctx);
        let output = JsonEncoderBuilder::new().serialize(&input, encode_ctx.borrow())?;
        assert_eq!(expected, output);
        let input_stream = input.start_stream(encode_ctx.borrow())?;
        let mut decode_ctx = OwnedContext::new();
        let mut shared_de_ctx = SharedArcDeserializeContext::default();
        decode_ctx.insert_mut(&mut shared_de_ctx);
        let output = JsonDecoderBuilder::new(output.as_bytes()).deserialize(decode_ctx.borrow())?;
        Ok(Tester {
            shared_ser_ctx,
            input,
            input_stream,
            shared_de_ctx,
            output,
        })
    }
    pub fn input_mut(&mut self) -> &mut T {
        &mut self.input
    }
    pub fn input(&self) -> &T {
        &self.input
    }
    pub fn output(&self) -> &T {
        &self.output
    }
    #[track_caller]
    pub fn next(&mut self, expected: &str) -> anyhow::Result<()> {
        let mut encode_ctx = OwnedContext::new();
        encode_ctx.insert_mut(&mut self.shared_ser_ctx);
        let output = JsonEncoderBuilder::new().with(|e| {
            self.input
                .serialize_update(&mut self.input_stream, e, encode_ctx.borrow())
        })?;
        assert_eq!(output, expected);
        let mut decode_ctx = OwnedContext::new();
        decode_ctx.insert_mut(&mut self.shared_de_ctx);
        JsonDecoderBuilder::new(output.as_bytes())
            .with(|d| self.output.deserialize_update(d, decode_ctx.borrow()))?;
        Ok(())
    }
}
