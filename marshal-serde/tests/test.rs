#![deny(unused_must_use)]

use marshal::context::OwnedContext;
use marshal_bin::decode::full::BinDecoderBuilder;
use marshal_bin::decode::BinDecoderSchema;
use marshal_bin::encode::full::BinEncoderBuilder;
use marshal_bin::encode::BinEncoderSchema;
use marshal_serde::WithSerde;
use std::collections::HashMap;
use std::fmt::Debug;

fn test_round_trip<T: Debug + Eq + serde::Serialize + for<'de> serde::Deserialize<'de>>(
    input: &T,
) -> anyhow::Result<()> {
    let mut en_schema = BinEncoderSchema::new();
    let encoded = BinEncoderBuilder::new(&mut en_schema)
        .serialize(&WithSerde::new(input), OwnedContext::new().borrow())?;
    println!("{:?}", encoded);
    let mut de_schema = BinDecoderSchema::new();
    let decoded = BinDecoderBuilder::new(&encoded, &mut de_schema)
        .deserialize::<WithSerde<T>>(OwnedContext::new().borrow())?
        .into_inner();
    assert_eq!(input, &decoded);
    Ok(())
}

#[test]
fn test_prim() -> anyhow::Result<()> {
    test_round_trip(&())?;
    test_round_trip(&4u8)?;
    test_round_trip(&8u16)?;
    test_round_trip(&15u32)?;
    test_round_trip(&16u64)?;
    test_round_trip(&23u128)?;
    test_round_trip(&42usize)?;
    test_round_trip(&4i8)?;
    test_round_trip(&8i16)?;
    test_round_trip(&15i32)?;
    test_round_trip(&16i64)?;
    test_round_trip(&23i128)?;
    test_round_trip(&42isize)?;
    test_round_trip(&(4, 8, 15, 16, 32, 42))?;
    test_round_trip(&vec![4, 8, 15, 16, 32, 42])?;

    #[derive(Eq, PartialEq, Debug, serde::Serialize, serde::Deserialize)]
    struct UnitStruct;
    test_round_trip(&UnitStruct)?;

    #[derive(Eq, PartialEq, Debug, serde::Serialize, serde::Deserialize)]
    struct NewTypeStruct(u8);
    test_round_trip(&NewTypeStruct(4))?;

    #[derive(Eq, PartialEq, Debug, serde::Serialize, serde::Deserialize)]
    struct TupleStruct(u8, u16);
    test_round_trip(&TupleStruct(4, 8))?;

    #[derive(Eq, PartialEq, Debug, serde::Serialize, serde::Deserialize)]
    struct Struct {
        x: u8,
        y: u16,
        z: u32,
    }
    test_round_trip(&Struct { x: 4, y: 8, z: 15 })?;

    #[derive(Eq, PartialEq, Debug, serde::Serialize, serde::Deserialize)]
    enum Enum {
        UnitVariant,
        NewtypeVariant(u8),
        TupleVariant(u8, u16),
        StructVariant { x: u8, y: u16, z: u32 },
    }
    test_round_trip(&Enum::UnitVariant)?;
    test_round_trip(&Enum::NewtypeVariant(4))?;
    test_round_trip(&Enum::TupleVariant(4, 8))?;
    test_round_trip(&Enum::StructVariant { x: 4, y: 8, z: 15 })?;

    test_round_trip(&"hello".to_string())?;

    test_round_trip(
        &[(4, 8), (15, 16), (23, 42)]
            .into_iter()
            .collect::<HashMap<u8, u8>>(),
    )?;

    Ok(())
}
