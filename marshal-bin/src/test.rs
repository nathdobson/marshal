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
#[track_caller]
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

fn test_transmute<
    T1: Debug + for<'s> Serialize<BinWriter<'s>>,
    T2: Debug + Eq + for<'de, 's> Deserialize<'de, BinParser<'de, 's>>,
>(
    input: T1,
    output: T2,
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
    let f = T2::deserialize(p.build(), &mut c)?;
    assert_eq!(output, f);
    Ok(())
}

#[test]
fn test_unit() -> anyhow::Result<()> {
    test_round_trip((), &[0])?;
    Ok(())
}
#[test]
fn test_int() -> anyhow::Result<()> {
    test_round_trip(123u8, &[7, 123])?;
    Ok(())
}

#[test]
fn test_tuple() -> anyhow::Result<()> {
    test_round_trip((123u8, 124u16), &[15, 2, 7, 123, 8, 124])?;
    Ok(())
}

#[test]
fn test_string() -> anyhow::Result<()> {
    test_round_trip("abc".to_string(), &[22, 3, b'a', b'b', b'c'])?;
    Ok(())
}

#[test]
fn test_struct() -> anyhow::Result<()> {
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
    test_round_trip(
        vec![Foo { abc: 123, xyz: 124 }, Foo { abc: 125, xyz: 126 }],
        &[
            19, 2, //
            21, 2, 3, b'a', b'b', b'c', 3, b'x', b'y', b'z', //
            16, 0, 7, 123, 9, 124, //
            16, 0, 7, 125, 9, 126,
        ],
    )?;

    #[derive(Serialize, Deserialize, Debug, Eq, PartialEq)]
    struct OrderOne {
        x1: u32,
        x2: u64,
    }

    #[derive(Serialize, Deserialize, Debug, Eq, PartialEq)]
    struct OrderTwo {
        x2: u64,
        x1: u32,
    }

    #[derive(Serialize, Deserialize, Debug, Eq, PartialEq)]
    struct OrderThree {
        x1: u32,
        x2: u64,
    }

    test_transmute(
        (OrderOne { x1: 51, x2: 52 }, OrderOne { x1: 53, x2: 54 }),
        (OrderOne { x1: 51, x2: 52 }, OrderTwo { x1: 53, x2: 54 }),
        &[
            15, 2, //
            21, 2, 2, b'x', b'1', 2, b'x', b'2', //
            16, 0, 9, 51, 10, 52, //
            16, 0, 9, 53, 10, 54, //
        ],
    )?;

    test_transmute(
        (OrderOne { x1: 51, x2: 52 }, OrderThree { x1: 53, x2: 54 }),
        (OrderOne { x1: 51, x2: 52 }, OrderOne { x1: 53, x2: 54 }),
        &[
            15, 2, //
            21, 2, 2, b'x', b'1', 2, b'x', b'2', //
            16, 0, 9, 51, 10, 52, //
            16, 0, 9, 53, 10, 54, //
        ],
    )?;
    Ok(())
}
#[test]
fn test_enum() -> anyhow::Result<()> {
    #[derive(Serialize, Deserialize, Debug, Eq, PartialEq)]
    pub enum Enum1 {
        V0,
        V1,
    }
    #[derive(Serialize, Deserialize, Debug, Eq, PartialEq)]
    pub enum Enum2 {
        V2,
        V3,
    }
    test_round_trip(
        (Enum1::V0, Enum1::V1, Enum2::V2, Enum2::V3),
        &[
            15, 4, //
            21, 2, 2, b'V', b'0', 2, b'V', b'1', //
            18, 0, 0, 0, //
            18, 0, 1, 0, //
            21, 2, 2, b'V', b'2', 2, b'V', b'3', //
            18, 1, 0, 0, //
            18, 1, 1, 0, //
        ],
    )?;
    Ok(())
}
