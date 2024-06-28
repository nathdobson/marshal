#![deny(unused_must_use)]

use std::sync;
use std::sync::Arc;

use marshal_derive::{Deserialize, Serialize};
use marshal_update::hash_map::UpdateHashMap;
use marshal_update::tester::Tester;
use marshal_update::{DeserializeUpdate, SerializeStream, SerializeUpdate};

#[test]
fn test_simple() -> anyhow::Result<()> {
    let mut tester = Tester::new(4u8, "4")?;
    tester.next("null")?;
    *tester.input_mut() = 8;
    tester.next("8")?;
    Ok(())
}

#[test]
fn test_unit_struct() -> anyhow::Result<()> {
    #[derive(Serialize, Deserialize, DeserializeUpdate, SerializeStream, SerializeUpdate)]
    struct Foo;
    let mut tester = Tester::new(
        Foo,
        r#"null"#,
    )?;
    tester.next("null")?;
    Ok(())
}

#[test]
fn test_tuple_struct() -> anyhow::Result<()> {
    #[derive(Serialize, Deserialize, DeserializeUpdate, SerializeStream, SerializeUpdate)]
    struct Foo(u8, u16);
    let (mut tester) = Tester::new(
        Foo(4, 8),
        r#"[
  4,
  8
]"#,
    )?;
    tester.next(r#"[
  null,
  null
]"#)?;
    tester.input_mut().0 = 15;
    tester.next(
        r#"[
  15,
  null
]"#,
    )?;
    assert_eq!(tester.output().0, 15);
    Ok(())
}

#[test]
fn test_struct() -> anyhow::Result<()> {
    #[derive(Serialize, Deserialize, DeserializeUpdate, SerializeStream, SerializeUpdate)]
    struct Foo {
        x: u8,
        y: u16,
    }
    let (mut tester) = Tester::new(
        Foo { x: 4, y: 8 },
        r#"{
  "x": 4,
  "y": 8
}"#,
    )?;
    tester.next(
        r#"{
  "x": null,
  "y": null
}"#,
    )?;
    tester.input_mut().x = 15;
    tester.next(
        r#"{
  "x": 15,
  "y": null
}"#,
    )?;
    assert_eq!(tester.output().x, 15);
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
