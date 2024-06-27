use marshal::context::OwnedContext;
use marshal_json::decode::full::JsonDecoderBuilder;
use marshal_json::encode::full::JsonEncoderBuilder;

use crate::{StructCons, StructNil};

#[test]
fn test_json() -> anyhow::Result<()> {
    type Test0 = StructNil<"Test">;
    type Test1 = StructCons<"u8", u8, Test0>;
    type Test2 = StructCons<"u16", u16, Test1>;
    let start = Test2::new(10, Test1::new(20, Test0::new()));
    let output = JsonEncoderBuilder::new().serialize(&start, OwnedContext::new().borrow())?;
    assert_eq!("{\n  \"u16\": 10,\n  \"u8\": 20\n}", output);
    let input = JsonDecoderBuilder::new(output.as_bytes())
        .deserialize::<Test2>(OwnedContext::new().borrow())?;
    assert_eq!(input, start);
    Ok(())
}
