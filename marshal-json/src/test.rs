use std::fmt::Debug;

use marshal::context::Context;
use marshal::de::Deserialize;
use marshal::ser::Serialize;
use marshal_derive::{Deserialize, Serialize};

use crate::parse::full::{JsonParser, JsonParserBuilder};
use crate::write::full::{JsonWriter, JsonWriterBuilder};
use crate::write::SimpleJsonWriter;

#[derive(Serialize, Deserialize, PartialEq, Eq, Debug)]
struct Foo {}

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
    test_round_trip(Foo {}, "{}")?;
    Ok(())
}
