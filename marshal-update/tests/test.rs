#![deny(unused_must_use)]

use marshal_update::tree::json::JsonSerializeStream;
use marshal_update::tree::Tree;
use std::sync::Arc;


#[test]
fn test() -> anyhow::Result<()> {
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
    assert_eq!(stream.next()?, r#"{
  0: 8
}"#);


    Ok(())
}
