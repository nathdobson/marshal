use crate::read::full::{BinParser, BinParserBuilder};
use crate::read::BinParserSchema;
use crate::write::full::{BinWriter, BinWriterBuilder};
use crate::write::BinWriterSchema;
use crate::VU128_MAX_PADDING;
use marshal::context::Context;
use marshal::de::Deserialize;
use marshal::ser::Serialize;
use marshal_derive::{Deserialize, Serialize};
use std::fmt::Debug;

fn test_round_trip<
    T: Debug
        + Eq
        + for<'s> Serialize<BinWriter<'s>>
        + for<'de, 's> Deserialize<'de, BinParser<'de, 's>>,
>(
    input: T,
    expected: &[u8],
) -> anyhow::Result<()> {
    println!("{:?}", input);
    let mut writer_schema = BinWriterSchema::new();
    let mut w = BinWriterBuilder::new(&mut writer_schema);
    let mut c = Context::new();
    input.serialize(w.build(), &mut c)?;
    let found = w.end()?;
    assert_eq!(&found[0..found.len() - VU128_MAX_PADDING], expected);
    let mut parser_schema = BinParserSchema::new();
    let mut p = BinParserBuilder::new(&found, &mut parser_schema);
    let f = T::deserialize(p.build(), &mut c)?;
    assert_eq!(input, f);
    Ok(())
}

#[test]
fn test_rt() -> anyhow::Result<()> {
    test_round_trip((), &[0])?;
    test_round_trip(123u8, &[7, 123])?;
    test_round_trip((123u8, 124u16), &[15, 2, 7, 123, 8, 124])?;
    test_round_trip("abc".to_string(), &[22, 3, b'a', b'b', b'c'])?;
    #[derive(Serialize, Deserialize, Debug, Eq, PartialEq)]
    struct Foo {
        abc: u8,
        xyz: u32,
    }
    test_round_trip(
        Foo { abc: 123, xyz: 124 },
        &[
            21, 2, 3, b'a', b'b', b'c', 3, b'x', b'y', b'z', //
            16, 0, 7, 123, 9, 124,
        ],
    )?;

    // #[derive(Serialize, Deserialize, Debug, Eq, PartialEq)]
    // enum Bar {
    //     V1,
    //     V2,
    // }
    // test_round_trip(Bar::V1)?;
    // test_round_trip(Bar::V2)?;
    Ok(())
}
