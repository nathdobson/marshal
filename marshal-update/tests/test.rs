#![deny(unused_must_use)]

use marshal::context::Context;
use marshal::encode::{AnyEncoder, Encoder};
use marshal::ser::Serialize;
use marshal::Serialize;
use marshal_update::ser::{SerializeStream, SerializeUpdate};
use marshal_update::tree::json::JsonSerializeStream;
use marshal_update::tree::Tree;
use std::sync;
use std::sync::Arc;

#[test]
fn test_simple() -> anyhow::Result<()> {
    let tree = Arc::new(Tree::new(4u8));
    let mut stream = JsonSerializeStream::new(tree.clone());
    assert_eq!(
        stream.next()?,
        r#"{
  "id": 0,
  "inner": 4
}"#
    );
    assert_eq!(stream.next()?, r#"{}"#);
    *tree.write() = 8;
    assert_eq!(
        stream.next()?,
        r#"{
  0: 8
}"#
    );
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
    assert_eq!(stream.next()?,r#"{
  0: [
    {
      "None": null
    },
    {
      "id": 1,
      "inner": 4
    }
  ]
}"#);

    Ok(())
}
