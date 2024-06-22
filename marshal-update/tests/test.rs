#![deny(unused_must_use)]

use marshal::context::Context;
use marshal::encode::{AnyEncoder, Encoder};
use marshal::ser::Serialize;
use marshal::Serialize;
use marshal_update::ser::{SerializeStream, SerializeUpdate};
use marshal_update::tree::json::{JsonDeserializeStream, JsonSerializeStream, SerializeUpdateJson};
use marshal_update::tree::Tree;
use std::sync;
use std::sync::Arc;
use marshal::de::Deserialize;
use marshal_json::decode::full::JsonDecoder;
use marshal_update::de::DeserializeUpdate;

struct Tester<T> {
    serializer: JsonSerializeStream<T>,
    deserializer: JsonDeserializeStream,
}

impl<
        T: 'static + Sync + Send + SerializeUpdateJson + for<'de> DeserializeUpdate<'de, JsonDecoder<'de>>,
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
    tester.next(r#"{
  0: 8
}"#)?;
    Ok(())
}

#[test]
fn test_strong_graph() -> anyhow::Result<()> {
    let tree1: Arc<Tree<Option<Arc<Tree<u8>>>>> = Arc::new(Tree::new(None));
    let tree2: Arc<Tree<u8>> = Arc::new(Tree::new(4u8));
    let mut stream = JsonSerializeStream::new(tree1.clone());
    assert_eq!(
        stream.next()?,
        r#"{
  "id": 0,
  "inner": {
    "None": null
  }
}"#
    );
    assert_eq!(stream.next()?, "{}");
    *tree1.write() = Some(tree2);
    assert_eq!(
        stream.next()?,
        r#"{
  0: {
    "id": 1,
    "inner": 4
  }
}"#
    );

    Ok(())
}

#[test]
fn test_weak_graph() -> anyhow::Result<()> {
    let tree1: Arc<Tree<(Option<sync::Weak<Tree<u8>>>, Option<Arc<Tree<u8>>>)>> =
        Arc::new(Tree::new((None, None)));
    let tree2: Arc<Tree<u8>> = Arc::new(Tree::new(4u8));
    let mut stream = JsonSerializeStream::new(tree1.clone());
    assert_eq!(
        stream.next()?,
        r#"{
  "id": 0,
  "inner": [
    null,
    null
  ]
}"#
    );
    assert_eq!(stream.next()?, "{}");
    tree1.write().0 = Some(Arc::downgrade(&tree2));
    assert_eq!(
        stream.next()?,
        r#"{
  0: [
    1,
    null
  ]
}"#
    );
    tree1.write().1 = Some(tree2);
    assert_eq!(
        stream.next()?,
        r#"{
  0: [
    {
      "None": null
    },
    {
      "id": 1,
      "inner": 4
    }
  ]
}"#
    );

    Ok(())
}
