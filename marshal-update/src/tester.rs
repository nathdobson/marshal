use crate::de::DeserializeUpdate;
use crate::ser::{SerializeStream, SerializeUpdate};
use crate::tree::json::{JsonDeserializeStream, JsonSerializeStream, SerializeUpdateJson};
use crate::tree::Tree;
use marshal::context::{Context, OwnedContext};
use marshal_json::decode::full::{JsonDecoder, JsonDecoderBuilder};
use marshal_json::encode::full::{JsonEncoder, JsonEncoderBuilder};
use std::sync::Arc;

pub struct Tester<T: SerializeStream> {
    input: T,
    input_stream: T::Stream,
    output: T,
}

impl<
        T: SerializeStream
            + SerializeUpdate<JsonEncoder>
            + for<'de> DeserializeUpdate<'de, JsonDecoder<'de>>,
    > Tester<T>
{
    pub fn new(input: T, expected: &str) -> anyhow::Result<Self> {
        let mut encode_ctx = OwnedContext::new();
        let output = JsonEncoderBuilder::new().serialize(&input, encode_ctx.borrow())?;
        assert_eq!(expected, output);
        let input_stream = input.start_stream(encode_ctx.borrow())?;
        let mut decode_ctx = OwnedContext::new();
        let output = JsonDecoderBuilder::new(output.as_bytes()).deserialize(decode_ctx.borrow())?;
        Ok(Tester {
            input,
            input_stream,
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
        let output = JsonEncoderBuilder::new().with(|e| {
            self.input
                .serialize_update(&mut self.input_stream, e, encode_ctx.borrow())
        })?;
        assert_eq!(output, expected);
        let mut decode_ctx = OwnedContext::new();
        JsonDecoderBuilder::new(output.as_bytes())
            .with(|d| self.output.deserialize_update(d, decode_ctx.borrow()))?;
        Ok(())
    }
}

pub struct SharedTester<T> {
    serializer: JsonSerializeStream<T>,
    deserializer: JsonDeserializeStream,
}

impl<
        T: 'static
            + Sync
            + Send
            + SerializeUpdateJson
            + for<'de> DeserializeUpdate<'de, JsonDecoder<'de>>,
    > SharedTester<T>
{
    #[track_caller]
    pub fn new(value: Arc<Tree<T>>, expected: &str) -> anyhow::Result<(Self, Arc<Tree<T>>)> {
        let mut serializer = JsonSerializeStream::new(value);
        let start = serializer.next()?;
        assert_eq!(start, expected);
        let (deserializer, output) = JsonDeserializeStream::new(start.as_bytes())?;
        Ok((
            SharedTester {
                serializer,
                deserializer,
            },
            output,
        ))
    }
    #[track_caller]
    pub fn next(&mut self, expected: &str) -> anyhow::Result<()> {
        let message = self.serializer.next()?;
        assert_eq!(message, expected);
        self.deserializer.next(message.as_bytes())?;
        Ok(())
    }
}
