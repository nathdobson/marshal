#![deny(unused_must_use)]

use std::sync;
use std::sync::Arc;

use marshal_derive::{Deserialize, Serialize};
use marshal_update::{DeserializeUpdate, SerializeStream, SerializeUpdate};
use marshal_update::hash_map::UpdateHashMap;
use marshal_update::tester::{SharedTester, Tester};
use marshal_update::tree::{Forest, Tree};

#[test]
fn test_simple() -> anyhow::Result<()> {
    let mut tester = Tester::new(4u8, "4")?;
    tester.next("null")?;
    *tester.input_mut() = 8;
    tester.next("8")?;
    Ok(())
}

#[test]
fn test_simple_graph() -> anyhow::Result<()> {
    let mut forest = Forest::new();
    let input = forest.add(4u8);
    let (mut tester, output) = SharedTester::new(
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
    let mut forest = Forest::new();
    let input: Arc<Tree<Option<Arc<Tree<u8>>>>> = forest.add(None);
    let inner: Arc<Tree<u8>> = forest.add(4u8);
    let (mut tester, output) = SharedTester::new(
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
    let mut forest = Forest::new();
    let input: Arc<Tree<(Option<sync::Weak<Tree<u8>>>, Option<Arc<Tree<u8>>>)>> =
        forest.add((None, None));
    let inner: Arc<Tree<u8>> = forest.add(4u8);
    let (mut tester, output) = SharedTester::new(
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
    assert_eq!(*output.read().1.as_ref().unwrap().read(), 4);
    Ok(())
}

#[test]
fn test_unit_struct() -> anyhow::Result<()> {
    #[derive(Serialize, Deserialize, DeserializeUpdate, SerializeStream, SerializeUpdate)]
    struct Foo;
    let mut forest = Forest::new();
    let input: Arc<Tree<Foo>> = forest.add(Foo);
    let (mut tester, _) = SharedTester::new(
        input.clone(),
        r#"{
  "id": 0,
  "inner": []
}"#,
    )?;
    tester.next("{}")?;
    Ok(())
}

#[test]
fn test_tuple_struct() -> anyhow::Result<()> {
    #[derive(Serialize, Deserialize, DeserializeUpdate, SerializeStream, SerializeUpdate)]
    struct Foo(u8, u16);
    let mut forest = Forest::new();
    let input: Arc<Tree<Foo>> = forest.add(Foo(4, 8));
    let (mut tester, output) = SharedTester::new(
        input.clone(),
        r#"{
  "id": 0,
  "inner": [
    4,
    8
  ]
}"#,
    )?;
    tester.next("{}")?;
    input.write().0 = 15;
    tester.next(
        r#"{
  "0": [
    15,
    null
  ]
}"#,
    )?;
    assert_eq!(output.read().0, 15);
    Ok(())
}

#[test]
fn test_struct() -> anyhow::Result<()> {
    #[derive(Serialize, Deserialize, DeserializeUpdate, SerializeStream, SerializeUpdate)]
    struct Foo {
        x: u8,
        y: u16,
    }
    let mut forest = Forest::new();
    let input: Arc<Tree<Foo>> = forest.add(Foo { x: 4, y: 8 });
    let (mut tester, output) = SharedTester::new(
        input.clone(),
        r#"{
  "id": 0,
  "inner": {
    "x": 4,
    "y": 8
  }
}"#,
    )?;
    tester.next("{}")?;
    input.write().x = 15;
    tester.next(
        r#"{
  "0": {
    "x": 15,
    "y": null
  }
}"#,
    )?;
    assert_eq!(output.read().x, 15);
    Ok(())
}

#[test]
fn test_map() -> anyhow::Result<()> {
    let mut map = UpdateHashMap::new();
    map.insert(4, 8);
    let mut tester = Tester::new(
        map,
        r#"{
  "4": 8
}"#,
    )?;
    tester.next("{}")?;
    tester.input_mut().insert(15, 16);
    tester.next(
        r#"{
  "15": 16
}"#,
    )?;
    assert_eq!(*tester.output().get(&15).unwrap(), 16);
    tester.input_mut().remove(&4);
    tester.next(
        r#"{
  "4": null
}"#,
    )?;
    assert!(
        tester.output().get(&4).is_none(),
        "{:?} {:?}",
        tester.input(),
        tester.output()
    );
    tester.input_mut().insert(15, 23);

    tester.next(
        r#"{
  "15": {
    "Some": 23
  }
}"#,
    )?;
    assert_eq!(*tester.output().get(&15).unwrap(), 23);
    Ok(())
}

