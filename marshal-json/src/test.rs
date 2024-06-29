use std::fmt::Debug;

use marshal::context::OwnedContext;
use marshal::de::Deserialize;
use marshal::ser::Serialize;
use marshal_derive::{Deserialize, Serialize};

use crate::decode::full::JsonDecoderBuilder;
use crate::encode::full::{JsonEncoderBuilder, JsonEncoder};
use crate::JsonDecoder;

#[track_caller]
pub fn test_round_trip<
    T: Debug + Eq + Serialize<JsonEncoder> + Deserialize<JsonDecoder>,
>(
    input: T,
    expected: &str,
) -> anyhow::Result<()> {
    println!("{:?}", input);
    let mut w = JsonEncoderBuilder::new();
    let mut c = OwnedContext::new();
    input.serialize(w.build(), c.borrow())?;
    let found = w.end()?;
    assert_eq!(expected.trim_start(), found);
    let mut p = JsonDecoderBuilder::new(found.as_bytes());
    let f = T::deserialize(p.build(), c.borrow())?;
    p.end()?;
    assert_eq!(input, f);
    Ok(())
}

#[test]
fn test_rt() -> anyhow::Result<()> {
    #[derive(Serialize, Deserialize, PartialEq, Eq, Debug)]
    struct Struct0 {}
    #[derive(Serialize, Deserialize, PartialEq, Eq, Debug)]
    struct Struct1 {
        x: u32,
    }
    #[derive(Serialize, Deserialize, PartialEq, Eq, Debug)]
    struct Struct2 {
        x: u32,
        y: u32,
    }

    test_round_trip(Struct0 {}, "{}")?;
    test_round_trip(Struct1 { x: 123 }, "{\n  \"x\": 123\n}")?;
    test_round_trip(
        Struct2 { x: 123, y: 234 },
        "{\n  \"x\": 123,\n  \"y\": 234\n}",
    )?;
    test_round_trip(Option::<!>::None, "null")?;
    test_round_trip(Some(Option::<!>::None), "{\n  \"None\": null\n}")?;
    test_round_trip(Some(Some(Option::<!>::None)), "{\n  \"Some\": null\n}")?;
    test_round_trip(
        Some(Some(Some(Option::<!>::None))),
        "{\n  \"Some\": {\n    \"None\": null\n  }\n}",
    )?;

    #[derive(Serialize, Deserialize, PartialEq, Eq, Debug)]
    enum Enum1 {
        A,
        B(u8, u16),
        C { x: u32, y: u32, z: u32 },
    }

    test_round_trip(
        vec![Enum1::A, Enum1::B(1, 2), Enum1::C { x: 3, y: 4, z: 5 }],
        r#"
[
  {
    "A": null
  },
  {
    "B": [
      1,
      2
    ]
  },
  {
    "C": {
      "x": 3,
      "y": 4,
      "z": 5
    }
  }
]"#,
    )?;

    Ok(())
}
