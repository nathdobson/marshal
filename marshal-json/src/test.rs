use std::fmt::Debug;

use marshal::context::Context;
use marshal::de::Deserialize;
use marshal::ser::Serialize;
use marshal_derive::{Deserialize, Serialize};

use crate::parse::full::{JsonParser, JsonParserBuilder};
use crate::write::full::{JsonWriter, JsonWriterBuilder};

fn test_round_trip<
    T: Debug + Eq + Serialize<JsonWriter> + for<'de> Deserialize<'de, JsonParser<'de>>,
>(
    input: T,
    expected: &str,
) -> anyhow::Result<()> {
    let mut w = JsonWriterBuilder::new();
    let mut c = Context::new();
    input.serialize(w.build(), &mut c)?;
    let found = w.end()?;
    assert_eq!(expected, found);
    let mut p = JsonParserBuilder::new(found.as_bytes());
    let f = T::deserialize(p.build(), &mut c)?;
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
    Ok(())
}
