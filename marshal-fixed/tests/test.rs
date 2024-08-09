#![feature(never_type)]

use marshal::context::OwnedContext;
use marshal::de::Deserialize;
use marshal::ser::Serialize;
use marshal::{Deserialize, Serialize};
use marshal_fixed::decode::full::{FixedDecoder, FixedDecoderBuilder};
use marshal_fixed::encode::full::{FixedEncoder, FixedEncoderBuilder};
use marshal_vu128::VU128_PADDING;
use std::fmt::Debug;
use std::time::{Duration, Instant, SystemTime};

#[track_caller]
pub fn test_round_trip<T: Debug + Eq + Serialize<FixedEncoder> + Deserialize<FixedDecoder>>(
    input: T,
    expected: &[u8],
) -> anyhow::Result<()> {
    println!("{:?}", input);
    let mut w = FixedEncoderBuilder::new();
    let mut c = OwnedContext::new();
    input.serialize(w.build(), c.borrow())?;
    let found = w.end()?;
    if !expected.is_empty() {
        assert_eq!(expected, &found[..found.len() - VU128_PADDING]);
    }
    let mut p = FixedDecoderBuilder::new(&found);
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

    test_round_trip(u8::MAX, &[])?;
    test_round_trip(u16::MAX, &[])?;
    test_round_trip(u32::MAX, &[])?;
    test_round_trip(u64::MAX, &[])?;
    test_round_trip(u128::MAX, &[])?;
    test_round_trip(Struct0 {}, &[])?;
    test_round_trip(Struct1 { x: 123 }, &[123])?;
    test_round_trip(Struct2 { x: 123, y: 23 }, &[123, 23])?;
    test_round_trip(Option::<!>::None, &[0])?;
    test_round_trip(Some(Option::<!>::None), &[1, 0])?;
    test_round_trip(Some(Some(Option::<!>::None)), &[1, 1, 0])?;
    test_round_trip(Some(Some(Some(Option::<!>::None))), &[1, 1, 1, 0])?;

    #[derive(Serialize, Deserialize, PartialEq, Eq, Debug)]
    enum Enum1 {
        A,
        B(u8, u16),
        C { x: u32, y: u32, z: u32 },
    }

    test_round_trip(
        vec![Enum1::A, Enum1::B(1, 2), Enum1::C { x: 3, y: 4, z: 5 }],
        &[3, 0, 1, 1, 2, 2, 3, 4, 5],
    )?;
    test_round_trip("\u{0000}".to_string(), &[1, 0])?;
    test_round_trip(vec![0u8], &[1, 0])?;
    test_round_trip(vec![0u8, 0u8], &[2, 0, 0])?;
    test_round_trip(vec![0u8, 0u8, 0u8], &[3, 0, 0, 0])?;
    test_round_trip(vec![0u8, 0u8, 0u8, 0u8], &[4, 0, 0, 0, 0])?;
    #[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
    struct Renamed {
        #[marshal(rename = "type")]
        typ: u8,
    }
    test_round_trip(Renamed { typ: 1 }, &[1])?;
    test_round_trip(Ok::<u8, u16>(1), &[0, 1])?;
    test_round_trip((), &[])?;
    test_round_trip((0u8,), &[0])?;
    test_round_trip((0u8, 1u8), &[0, 1])?;
    test_round_trip((0u8, 1u8, 2u8), &[0, 1, 2])?;
    test_round_trip((0u8, 1u8, 2u8, 3u8), &[0, 1, 2, 3])?;
    test_round_trip((0u8, 1u8, 2u8, 3u8, 4u8), &[0, 1, 2, 3, 4])?;
    // test_round_trip(Instant::now(), &[])?;
    test_round_trip(Duration::from_secs(1), &[])?;
    test_round_trip(Duration::from_secs(u64::MAX), &[])?;
    test_round_trip(1722628047u64, &[])?;
    test_round_trip(352217000u32, &[])?;
    test_round_trip((1722628047u64, 352217000u32), &[])?;
    test_round_trip(SystemTime::now(), &[])?;

    Ok(())
}
