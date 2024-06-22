#![deny(unused_must_use)]

use std::sync;
use std::sync::Arc;

use marshal_json::decode::full::JsonDecoder;
use marshal_update::de::DeserializeUpdate;
use marshal_update::tree::json::{JsonDeserializeStream, JsonSerializeStream, SerializeUpdateJson};
use marshal_update::tree::Tree;

struct Tester<T> {
    serializer: JsonSerializeStream<T>,
    deserializer: JsonDeserializeStream,
}

impl<
        T: 'static
            + Sync
            + Send
            + SerializeUpdateJson
            + for<'de> DeserializeUpdate<'de, JsonDecoder<'de>>,
    > Tester<T>
{
    pub fn new(value: Arc<Tree<T>>, expected: &str) -> anyhow::Result<(Self, Arc<Tree<T>>)> {
        let mut serializer = JsonSerializeStream::new(value);
        let start = serializer.next()?;
        assert_eq!(start, expected);
        let (deserializer, output) = JsonDeserializeStream::new(start.as_bytes())?;
        Ok((
            Tester {
                serializer,
                deserializer,
            },
            output,
        ))
    }
    pub fn next(&mut self, expected: &str) -> anyhow::Result<()> {
        let message = self.serializer.next()?;
        assert_eq!(message, expected);
        self.deserializer.next(message.as_bytes())?;
        Ok(())
    }
}

#[test]
fn test_simple() -> anyhow::Result<()> {
    let input = Arc::new(Tree::new(4u8));
    let (mut tester, output) = Tester::new(
        input.clone(),
        r#"{
  "id": 0,
  "inner": 4
}"#,
    )?;
    tester.next("{}")?;
    assert_eq!(*output.read(), 4);
    *input.write() = 8;
    tester.next(
        r#"{
  "0": 8
}"#,
    )?;
    assert_eq!(*output.read(), 8);
    Ok(())
}

#[test]
fn test_strong_graph() -> anyhow::Result<()> {
    let input: Arc<Tree<Option<Arc<Tree<u8>>>>> = Arc::new(Tree::new(None));
    let inner: Arc<Tree<u8>> = Arc::new(Tree::new(4u8));
    let (mut tester, output) = Tester::new(
        input.clone(),
        r#"{
  "id": 0,
  "inner": {
    "None": null
  }
}"#,
    )?;
    tester.next("{}")?;
    *input.write() = Some(inner);
    tester.next(
        r#"{
  "0": {
    "id": 1,
    "inner": 4
  }
}"#,
    )?;
    assert_eq!(*output.read().as_ref().unwrap().read(), 4);
    Ok(())
}

#[test]
fn test_weak_graph() -> anyhow::Result<()> {
    let input: Arc<Tree<(Option<sync::Weak<Tree<u8>>>, Option<Arc<Tree<u8>>>)>> =
        Arc::new(Tree::new((None, None)));
    let inner: Arc<Tree<u8>> = Arc::new(Tree::new(4u8));
    let (mut tester, output) = Tester::new(
        input.clone(),
        r#"{
  "id": 0,
  "inner": [
    null,
    null
  ]
}"#,
    )?;
    tester.next("{}")?;
    input.write().0 = Some(Arc::downgrade(&inner));
    tester.next(
        r#"{
  "0": [
    1,
    null
  ]
}"#,
    )?;
    assert!(output.read().0.as_ref().unwrap().upgrade().is_none());
    input.write().1 = Some(inner);
    tester.next(
        r#"{
  "0": [
    {
      "None": null
    },
    {
      "id": 1,
      "inner": 4
    }
  ]
}"#,
    )?;
    assert_eq!(
        *output.read().0.as_ref().unwrap().upgrade().unwrap().read(),
        4
    );
    assert_eq!(
        *output.read().1.as_ref().unwrap().read(),
        4
    );
    Ok(())
}
