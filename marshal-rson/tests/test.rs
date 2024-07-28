#![deny(unused_must_use)]
#![feature(never_type)]

use marshal::context::OwnedContext;
use marshal::de::Deserialize;
use marshal::reexports::anyhow;
use marshal::ser::Serialize;
use marshal::{Deserialize, Serialize};
use marshal_rson::decode::full::{RsonDecoder, RsonDecoderBuilder};
use marshal_rson::encode::full::{RsonEncoder, RsonEncoderBuilder};
use ordered_float::OrderedFloat;
use std::collections::{BTreeMap, HashMap};
use std::fmt::Debug;

#[track_caller]
pub fn test_round_trip<T: Debug + PartialEq + Serialize<RsonEncoder> + Deserialize<RsonDecoder>>(
    input: T,
    expected: &str,
) -> anyhow::Result<()> {
    println!("{:?}", input);
    let mut w = RsonEncoderBuilder::new();
    let mut c = OwnedContext::new();
    input.serialize(w.build(), c.borrow())?;
    let found = w.end()?;
    assert_eq!(expected.trim_start(), found);
    let mut p = RsonDecoderBuilder::new(&found);
    let f = T::deserialize(p.build(), c.borrow())?;
    p.end()?;
    assert_eq!(input, f);
    Ok(())
}

#[test]
fn test() -> anyhow::Result<()> {
    test_round_trip((), r#"unit"#)?;
    test_round_trip(false, r#"false"#)?;
    test_round_trip(true, r#"true"#)?;
    test_round_trip(1u8, r#"u8 1"#)?;
    test_round_trip(2u16, r#"u16 2"#)?;
    test_round_trip(3u32, r#"u32 3"#)?;
    test_round_trip(4u64, r#"u64 4"#)?;
    test_round_trip(5u128, r#"u128 5"#)?;
    test_round_trip(6i8, r#"i8 6"#)?;
    test_round_trip(7i16, r#"i16 7"#)?;
    test_round_trip(8i32, r#"i32 8"#)?;
    test_round_trip(9i64, r#"i64 9"#)?;
    test_round_trip(10i128, r#"i128 10"#)?;
    test_round_trip(11f32, r#"f32 11"#)?;
    test_round_trip(12f64, r#"f64 12"#)?;
    test_round_trip(OrderedFloat(f32::NAN), r#"f32 NaN"#)?;
    test_round_trip(OrderedFloat(f32::INFINITY), r#"f32 inf"#)?;
    test_round_trip(OrderedFloat(f32::NEG_INFINITY), r#"f32 -inf"#)?;
    test_round_trip('a', r#"char 'a'"#)?;
    test_round_trip("ab'\"\n\\a".to_owned(), r#"string "ab'\"\n\\a""#)?;
    test_round_trip(Vec::<u8>::new(), r#"bytes """#)?;
    test_round_trip(vec![1u8], r#"bytes "AQ==""#)?;
    test_round_trip(vec![1u8, 2u8], r#"bytes "AQI=""#)?;
    test_round_trip(vec![1u8, 2u8, 3u8], r#"bytes "AQID""#)?;
    test_round_trip(vec![1u8, 2u8, 3u8, 4u8], r#"bytes "AQIDBA==""#)?;
    test_round_trip(Option::<!>::None, r#"none"#)?;
    test_round_trip(Some(Option::<!>::None), r#"some none"#)?;
    test_round_trip(Some(Some(Option::<!>::None)), r#"some some none"#)?;
    test_round_trip((1u8,), r#"(u8 1,)"#)?;
    test_round_trip(
        (1u8, 2u16),
        r#"
(
  u8 1,
  u16 2,
)"#,
    )?;
    test_round_trip(
        (1u8, 2u16, 3u32),
        r#"
(
  u8 1,
  u16 2,
  u32 3,
)"#,
    )?;

    #[derive(Eq, PartialEq, Debug, Serialize, Deserialize)]
    struct UnitStruct;
    test_round_trip(UnitStruct, r#"struct UnitStruct"#)?;

    #[derive(Eq, PartialEq, Debug, Serialize, Deserialize)]
    struct EmptyTupleStruct();
    test_round_trip(EmptyTupleStruct(), r#"struct EmptyTupleStruct()"#)?;

    #[derive(Eq, PartialEq, Debug, Serialize, Deserialize)]
    struct Tuple1Struct(u8);
    test_round_trip(
        Tuple1Struct(1u8),
        r#"
struct Tuple1Struct(u8 1)"#,
    )?;

    #[derive(Eq, PartialEq, Debug, Serialize, Deserialize)]
    struct Tuple2Struct(u8, u16);
    test_round_trip(
        Tuple2Struct(1u8, 2u16),
        r#"
struct Tuple2Struct(
  u8 1,
  u16 2,
)"#,
    )?;
    #[derive(Eq, PartialEq, Debug, Serialize, Deserialize)]
    struct Tuple3Struct(u8, u16, u32);
    test_round_trip(
        Tuple3Struct(1u8, 2u16, 3u32),
        r#"
struct Tuple3Struct(
  u8 1,
  u16 2,
  u32 3,
)"#,
    )?;

    #[derive(Eq, PartialEq, Debug, Serialize, Deserialize)]
    struct EmptyStruct {}
    test_round_trip(EmptyStruct {}, r#"struct EmptyStruct {}"#)?;

    #[derive(Eq, PartialEq, Debug, Serialize, Deserialize)]
    struct Struct1 {
        x: u8,
    }
    test_round_trip(Struct1 { x: 1u8 }, r#"struct Struct1 { x: u8 1 }"#)?;

    #[derive(Eq, PartialEq, Debug, Serialize, Deserialize)]
    struct Struct2 {
        x: u8,
        y: u16,
    }
    test_round_trip(
        Struct2 { x: 1u8, y: 2u16 },
        r#"
struct Struct2 {
  x: u8 1,
  y: u16 2,
}"#,
    )?;
    #[derive(Eq, PartialEq, Debug, Serialize, Deserialize)]
    struct Struct3 {
        x: u8,
        y: u16,
        z: u32,
    }
    test_round_trip(
        Struct3 { x: 1, y: 2, z: 3 },
        r#"
struct Struct3 {
  x: u8 1,
  y: u16 2,
  z: u32 3,
}"#,
    )?;

    #[derive(Eq, PartialEq, Debug, Serialize, Deserialize)]
    enum Enum {
        UnitVariant,
        TupleVariant0(),
        TupleVariant1(u8),
        TupleVariant2(u8, u16),
        StructVariant0 {},
        StructVariant1 { x: u8 },
        StructVariant2 { x: u8, y: u16 },
    }

    test_round_trip(Enum::UnitVariant, r#"enum Enum::UnitVariant"#)?;

    test_round_trip(Enum::TupleVariant0(), r#"enum Enum::TupleVariant0()"#)?;

    test_round_trip(
        Enum::TupleVariant1(1u8),
        r#"enum Enum::TupleVariant1(u8 1)"#,
    )?;

    test_round_trip(
        Enum::TupleVariant2(1u8, 2u16),
        r#"
enum Enum::TupleVariant2(
  u8 1,
  u16 2,
)"#,
    )?;

    test_round_trip(
        Enum::StructVariant0 {},
        r#"
enum Enum::StructVariant0 {}"#,
    )?;

    test_round_trip(
        Enum::StructVariant1 { x: 1 },
        r#"
enum Enum::StructVariant1 { x: u8 1 }"#,
    )?;

    test_round_trip(
        Enum::StructVariant2 { x: 1, y: 2 },
        r#"
enum Enum::StructVariant2 {
  x: u8 1,
  y: u16 2,
}"#,
    )?;

    test_round_trip(Vec::<!>::new(), r"[]")?;
    test_round_trip(vec![1u16], r"[u16 1]")?;
    test_round_trip(
        vec![1u16, 2u16],
        r"[
  u16 1,
  u16 2,
]",
    )?;

    test_round_trip(HashMap::<!, !>::new(), r"{}")?;
    test_round_trip(
        vec![(1u16, 2u16)]
            .into_iter()
            .collect::<BTreeMap<u16, u16>>(),
        r"{ u16 1: u16 2 }",
    )?;
    test_round_trip(
        vec![(1u16, 2u16), (3, 4)]
            .into_iter()
            .collect::<BTreeMap<u16, u16>>(),
        r"{
  u16 1: u16 2,
  u16 3: u16 4,
}",
    )?;
    #[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
    enum RenamedVariant {
        #[marshal(rename = r#"abc::<cd>::ef"#)]
        Variant,
    }
    test_round_trip(
        RenamedVariant::Variant,
        r#"enum RenamedVariant::<abc::<cd>::ef>"#,
    )?;
    Ok(())
}
