#![deny(unused_must_use)]

#![feature(arbitrary_self_types)]
use marshal_bin::bin_object;
use std::fmt::Debug;

use marshal_derive::{Deserialize, Serialize};
use marshal_json::json_object;
use marshal_object::{derive_box_object, derive_variant, AsDiscriminant};
use marshal_pointer::RawAny;
use marshal_update::object_map::ObjectMap;
use marshal_update::tester::Tester;

pub struct BoxFoo;
derive_box_object!(BoxFoo, Foo, bin_object, json_object);
pub trait Foo: 'static + Debug + RawAny + AsDiscriminant<BoxFoo> {}

derive_variant!(BoxFoo, A);
#[derive(Serialize, Deserialize, Debug, Eq, PartialEq, Ord, PartialOrd)]
struct A(u8);
impl Foo for A {}

derive_variant!(BoxFoo, B);
#[derive(Serialize, Deserialize, Debug, Eq, PartialEq, Ord, PartialOrd)]
struct B(u16);
impl Foo for B {}

#[test]
fn test() -> anyhow::Result<()> {
    let mut map = ObjectMap::<BoxFoo>::new();
    map.insert(Box::new(A(4)));
    let mut tester = Tester::new(
        map,
        r#"[
  {
    "test_object::A": [
      [
        4
      ]
    ]
  }
]"#,
    )?;
    assert_eq!(tester.output().get::<A>().unwrap().0, 4);
    tester.next("[]")?;
    tester.input_mut().insert(Box::new(B(8)));
    tester.next(r#"[
  {
    "test_object::B": [
      [
        8
      ]
    ]
  }
]"#)?;
    assert_eq!(tester.output().get::<B>().unwrap().0, 8);
    Ok(())
}
