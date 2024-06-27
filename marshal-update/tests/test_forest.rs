#![deny(unused_must_use)]

use marshal_update::forest::{Forest, ForestRoot, Tree};
use marshal_update::tester::Tester;
use std::sync::Arc;

#[test]
fn test_forest() -> anyhow::Result<()> {
    let forest = Forest::new();
    let tree = forest.add(4u8);
    let root = ForestRoot::new(forest, tree.clone());
    let mut tester = Tester::new(
        root,
        r#"{
  "id": 0,
  "inner": 4
}"#,
    )?;
    assert_eq!(4, *tester.output().forest().get(tester.output().root()));
    tester.next(
        r#"{
  "root": null,
  "trees": {}
}"#,
    )?;
    assert_eq!(4, *tester.output().forest().get(tester.output().root()));
    *tester.input_mut().forest_mut().get_mut(&tree) = 8;
    tester.next(
        r#"{
  "root": null,
  "trees": {
    "0": 8
  }
}"#,
    )?;
    assert_eq!(8, *tester.output().forest().get(tester.output().root()));
    Ok(())
}

#[test]
fn test_forest_insert() -> anyhow::Result<()> {
    let forest = Forest::new();
    let tree: Arc<Tree<Option<Arc<Tree<u8>>>>> = forest.add(None);
    let root = ForestRoot::new(forest, tree.clone());
    let mut tester = Tester::new(
        root,
        r#"{
  "id": 0,
  "inner": {
    "None": null
  }
}"#,
    )?;
    assert!(tester
        .output()
        .forest()
        .get(tester.output().root())
        .is_none());
    tester.next(
        r#"{
  "root": null,
  "trees": {}
}"#,
    )?;
    assert!(tester
        .output()
        .forest()
        .get(tester.output().root())
        .is_none());
    *tester.input_mut().forest_mut().get_mut(&tree) = Some(tester.input().forest().add(4));
    tester.next(
        r#"{
  "root": null,
  "trees": {
    "0": {
      "id": 1,
      "inner": 4
    }
  }
}"#,
    )?;
    {
        let o = tester.output();
        let r = o.root();
        let f = o.forest();
        assert_eq!(*f.get(f.get(r).as_ref().unwrap()), 4);
    }
    Ok(())
}
