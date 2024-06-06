use crate::parse::full::JsonParser;
use crate::write::full::JsonWriter;
use crate::write::SimpleJsonWriter;
use marshal::de::Deserialize;
use marshal::ser::Serialize;
use marshal_derive::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
struct Foo {}

fn test_round_trip<T: Serialize<JsonWriter> + for<'de> Deserialize<'de, JsonParser<'de>>>(
    input: T,
    expected: &str,
) {
    todo!();
}

#[test]
fn test_rt() {
    test_round_trip(Foo {}, "{}");
}
