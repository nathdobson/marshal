#![feature(trait_alias)]
#![feature(trait_upcasting)]
#![feature(const_trait_impl)]
#![feature(effects)]
#![feature(unsize)]

use std::any::{Any, TypeId};
use std::fmt::Debug;

use safe_once::sync::LazyLock;

use marshal::context::Context;
use marshal::de::Deserialize;
use marshal::decode::Decoder;
use marshal::encode::Encoder;
use marshal::ser::Serialize;
use marshal_bin::decode::full::BinDecoderBuilder;
use marshal_bin::decode::BinDecoderSchema;
use marshal_bin::encode::full::BinEncoderBuilder;
use marshal_bin::encode::BinEncoderSchema;
use marshal_bin::SerializeBin;
use marshal_bin::{DeserializeBin, VU128_MAX_PADDING};
use marshal_json::decode::full::JsonDecoderBuilder;
use marshal_json::encode::full::JsonEncoderBuilder;
use marshal_json::DeserializeJson;
use marshal_json::SerializeJson;
use marshal_object::de::{deserialize_object, DeserializeVariant};
use marshal_object::ser::serialize_object;
use marshal_object::{bin_format, json_format, ObjectDescriptor};
use marshal_object::{define_variant, derive_object, AsDiscriminant};
use marshal_object::{VariantRegistration, OBJECT_REGISTRY};

derive_object!(MyTrait, MyTraitParent, bin_format, json_format);
trait MyTrait: 'static + MyTraitParent + Debug + Any {}

impl MyTrait for u8 {}
impl MyTrait for u16 {}

define_variant!(u8, MyTrait);
define_variant!(u16, MyTrait);

#[track_caller]
pub fn json_round_trip<T: Debug + SerializeJson + for<'de> DeserializeJson<'de>>(
    input: &T,
    expected: &str,
) -> anyhow::Result<T> {
    println!("{:?}", input);
    let found = JsonEncoderBuilder::new().serialize(input, &mut Context::new())?;
    assert_eq!(expected.trim_start(), found);
    let output = JsonDecoderBuilder::new(found.as_bytes()).deserialize::<T>(&mut Context::new())?;
    Ok(output)
}

#[track_caller]
pub fn bin_round_trip<T: Debug + SerializeBin + for<'de> DeserializeBin<'de>>(
    input: &T,
    expected: &[&[u8]],
) -> anyhow::Result<T> {
    println!("{:?}", input);
    let found = BinEncoderBuilder::new(&mut BinEncoderSchema::new())
        .serialize(input, &mut Context::new())?;
    let compare = &found[0..found.len() - VU128_MAX_PADDING];
    assert!(expected.contains(&compare), "{:?}", compare);
    let output = BinDecoderBuilder::new(&found, &mut BinDecoderSchema::new())
        .deserialize::<T>(&mut Context::new())?;
    Ok(output)
}

#[test]
fn test_json() -> anyhow::Result<()> {
    let input = Box::new(42u8) as Box<dyn MyTrait>;
    let output = json_round_trip(
        &input,
        r#"{
  "u8": [
    42
  ]
}"#,
    )?;
    let output: u8 = *Box::<dyn Any>::downcast::<u8>(output).unwrap();
    assert_eq!(output, 42);
    Ok(())
}

#[test]
fn test_bin() -> anyhow::Result<()> {
    let input = Box::new(42u8) as Box<dyn MyTrait>;
    let output = bin_round_trip(
        &input,
        &[
            &[
                21, 2, 2, b'u', b'8', 3, b'u', b'1', b'6', //
                18, 0, 0, 17, 1, 7, 42,
            ],
            &[
                21, 2, 3, b'u', b'1', b'6', 2, b'u', b'8', //
                18, 0, 1, 17, 1, 7, 42,
            ],
            //
        ],
    )?;
    let output: u8 = *Box::<dyn Any>::downcast::<u8>(output).unwrap();
    assert_eq!(output, 42);
    Ok(())
}
