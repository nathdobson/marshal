use std::collections::BTreeMap;
use std::fmt::Debug;

use marshal::context::OwnedContext;
use marshal::de::Deserialize;
use marshal::ser::Serialize;
use marshal_derive::{Deserialize, Serialize};
use marshal_vu128::VU128_PADDING;
use crate::{BinDecoder};
use crate::decode::BinDecoderSchema;
use crate::decode::full::BinDecoderBuilder;
use crate::encode::BinEncoderSchema;
use crate::encode::full::{BinEncoder, BinEncoderBuilder};

#[track_caller]
fn test_round_trip<
    T: Debug + Eq + for<'s> Serialize<BinEncoder> + Deserialize<BinDecoder>,
>(
    input: T,
    expected: &[u8],
) -> anyhow::Result<()> {
    println!("{:?}", input);
    let mut encoder_schema = BinEncoderSchema::new();
    let mut w = BinEncoderBuilder::new(&mut encoder_schema);
    let mut c = OwnedContext::new();
    input.serialize(w.build(), c.borrow())?;
    let found = w.end()?;
    assert_eq!(&found[0..found.len() - VU128_PADDING], expected);
    let mut decoder_schema = BinDecoderSchema::new();
    let mut p = BinDecoderBuilder::new(&found, &mut decoder_schema);
    let f = T::deserialize(p.build(), c.borrow())?;
    p.end()?;
    assert_eq!(input, f);
    Ok(())
}

fn test_transmute<
    T1: Debug + for<'s> Serialize<BinEncoder>,
    T2: Debug + Eq + Deserialize<BinDecoder>,
>(
    input: T1,
    output: T2,
    expected: &[u8],
) -> anyhow::Result<()> {
    println!("{:?}", input);
    let mut writer_schema = BinEncoderSchema::new();
    let mut w = BinEncoderBuilder::new(&mut writer_schema);
    let mut c = OwnedContext::new();
    input.serialize(w.build(), c.borrow())?;
    let found = w.end()?;
    assert_eq!(&found[0..found.len() - VU128_PADDING], expected);
    let mut decoder_schema = BinDecoderSchema::new();
    let mut p = BinDecoderBuilder::new(&found, &mut decoder_schema);
    let f = T2::deserialize(p.build(), c.borrow())?;
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
fn test_vec() -> anyhow::Result<()> {
    test_round_trip(
        vec![Some(1), Some(2), Some(3)],
        &[19, 3, 26, 4, 2, 26, 4, 4, 26, 4, 6],
    )?;
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
fn test_unit_struct() -> anyhow::Result<()> {
    #[derive(Serialize, Deserialize, Debug, Eq, PartialEq)]
    struct Foo;
    test_round_trip(Foo, &[23])?;

    Ok(())
}

#[test]
fn test_tuple_struct() -> anyhow::Result<()> {
    #[derive(Serialize, Deserialize, Debug, Eq, PartialEq)]
    struct Foo(u8, u16, u32);
    test_round_trip(Foo(50, 51, 52), &[17, 3, 7, 50, 8, 51, 9, 52])?;

    Ok(())
}

#[test]
fn test_enum_variants() -> anyhow::Result<()> {
    #[derive(Serialize, Deserialize, Debug, Eq, PartialEq)]
    enum AllVariants {
        U,
        T(u8, u16),
        V { x: u32, y: u64, z: u128 },
    }
    test_round_trip(
        AllVariants::U,
        &[
            21, 3, 1, b'U', 1, b'T', 1, b'V', //
            18, 0, 0, 0, //
        ],
    )?;
    test_round_trip(
        AllVariants::T(51, 52),
        &[
            21, 3, 1, b'U', 1, b'T', 1, b'V', //
            18, 0, 1, 17, 2, 7, 51, 8, 52, //
        ],
    )?;
    test_round_trip(
        AllVariants::V {
            x: 53,
            y: 54,
            z: 55,
        },
        &[
            21, 3, 1, b'U', 1, b'T', 1, b'V', //
            21, 3, 1, b'x', 1, b'y', 1, b'z', //
            18, 0, 2, 16, 1, 9, 53, 10, 54, 11, 55, //
        ],
    )?;
    test_round_trip(
        vec![
            AllVariants::U,
            AllVariants::T(51, 52),
            AllVariants::V {
                x: 53,
                y: 54,
                z: 55,
            },
        ],
        &[
            19, 3, //
            21, 3, 1, b'U', 1, b'T', 1, b'V', //
            18, 0, 0, 0, //
            18, 0, 1, 17, 2, 7, 51, 8, 52, //
            21, 3, 1, b'x', 1, b'y', 1, b'z', //
            18, 0, 2, 16, 1, 9, 53, 10, 54, 11, 55, //
        ],
    )?;

    Ok(())
}

#[test]
fn test_enum_reorder() -> anyhow::Result<()> {
    #[derive(Serialize, Deserialize, Debug, Eq, PartialEq)]
    enum Foo1 {
        A1,
        A2,
    }
    #[derive(Serialize, Deserialize, Debug, Eq, PartialEq)]
    enum Foo2 {
        A2,
        A1,
    }
    test_transmute(
        Foo1::A1,
        Foo2::A1,
        &[
            21, 2, 2, b'A', b'1', 2, b'A', b'2', //
            18, 0, 0, 0,
        ],
    )?;
    test_transmute(
        Foo1::A2,
        Foo2::A2,
        &[
            21, 2, 2, b'A', b'1', 2, b'A', b'2', //
            18, 0, 1, 0,
        ],
    )?;

    Ok(())
}

#[test]
fn test_map() -> anyhow::Result<()> {
    test_round_trip::<BTreeMap<u8, u8>>(
        [(51, 52), (53, 54)].into_iter().collect(),
        &[20, 2, 7, 51, 7, 52, 7, 53, 7, 54],
    )?;
    Ok(())
}

#[test]
fn test_none() -> anyhow::Result<()> {
    test_round_trip::<Option<!>>(None, &[25])?;
    Ok(())
}

#[test]
fn test_some() -> anyhow::Result<()> {
    test_round_trip(Some(()), &[26, 0])?;
    Ok(())
}
