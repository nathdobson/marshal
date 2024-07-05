#![deny(unused_must_use)]

use std::sync::Arc;

use pretty_assertions::assert_eq;

use marshal_derive::{Deserialize, DeserializeUpdate, Serialize, SerializeStream, SerializeUpdate};
use marshal_pointer::Arcf;
use marshal_update::forest::forest::Forest;
use marshal_update::forest::forest::ForestRoot;
use marshal_update::forest::forest::Tree;
use marshal_update::tester::Tester;

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
    let tree: Arcf<Tree<Option<Arcf<Tree<u8>>>>> = forest.add(None);
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

#[test]
fn test_forest_list() -> anyhow::Result<()> {
    #[derive(Deserialize, DeserializeUpdate, Serialize, SerializeUpdate, SerializeStream)]
    struct Cons {
        head: Arcf<Tree<u8>>,
        tail: List,
    }
    type List = Option<Arcf<Tree<Cons>>>;
    let forest = Forest::new();
    let list = forest.add(Cons {
        head: forest.add(4),
        tail: None,
    });
    let root = ForestRoot::new(forest, list.clone());
    let mut tester = Tester::new(
        root,
        r#"{
  "id": 0,
  "inner": {
    "head": {
      "id": 1,
      "inner": 4
    },
    "tail": null
  }
}"#,
    )?;
    {
        let o = tester.output();
        let f = o.forest();
        let l = f.get(o.root());
        assert_eq!(*f.get(&l.head), 4);
        assert!(l.tail.is_none());
    }
    let tail = tester.input().forest().add(Cons {
        head: tester.input().forest().add(8),
        tail: None,
    });
    tester.input_mut().forest_mut().get_mut(&list).tail = Some(tail);
    tester.next(
        r#"{
  "root": null,
  "trees": {
    "0": {
      "head": null,
      "tail": {
        "id": 2,
        "inner": {
          "head": {
            "id": 3,
            "inner": 8
          },
          "tail": null
        }
      }
    }
  }
}"#,
    )?;
    {
        let o = tester.output();
        let f = o.forest();
        let l = f.get(o.root());
        assert_eq!(*f.get(&l.head), 4);
        let l = f.get(l.tail.as_ref().unwrap());
        assert_eq!(*f.get(&l.head), 8);
        assert!(l.tail.is_none());
    }
    Ok(())
}
