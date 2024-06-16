#![feature(trait_alias)]
#![feature(trait_upcasting)]
#![feature(const_trait_impl)]
#![feature(effects)]
#![feature(unsize)]
#![feature(coerce_unsized)]
#![feature(arbitrary_self_types)]

use std::any::Any;
use std::fmt::Debug;
use std::rc::Rc;

use safe_once::sync::LazyLock;

use marshal::{
    Deserialize, Serialize,
};
use marshal::context::Context;
use marshal::de::Deserialize;
use marshal::decode::Decoder;
use marshal::encode::Encoder;
use marshal::ser::rc::SerializeRcWeak;
use marshal::ser::Serialize;
use marshal_bin::{bin_object, VU128_MAX_PADDING};
use marshal_bin::decode::BinDecoderSchema;
use marshal_bin::decode::full::BinDecoderBuilder;
use marshal_bin::DeserializeBin;
use marshal_bin::encode::BinEncoderSchema;
use marshal_bin::encode::full::BinEncoderBuilder;
use marshal_bin::SerializeBin;
use marshal_json::{DeserializeJson, json_object};
use marshal_json::decode::full::JsonDecoderBuilder;
use marshal_json::encode::full::JsonEncoderBuilder;
use marshal_json::SerializeJson;
use marshal_object::{
    derive_arc_object, derive_box_object, derive_rc_object, derive_rc_weak_object, derive_variant,
    ObjectDescriptor,
};
use marshal_object::{AsDiscriminant, derive_object};
use marshal_object::{OBJECT_REGISTRY, VariantRegistration};
use marshal_object::de::{deserialize_object, DeserializeVariantForDiscriminant};
use marshal_object::ser::serialize_object;
use marshal_pointer::RawAny;
use marshal_shared::{derive_deserialize_arc_shared, derive_deserialize_arc_weak_shared, derive_deserialize_rc_shared, derive_deserialize_rc_weak_shared, derive_serialize_arc_shared, derive_serialize_arc_weak_shared, derive_serialize_rc_shared, derive_serialize_rc_weak_shared};

pub struct BoxMyTrait;
derive_box_object!(BoxMyTrait, MyTrait, bin_object, json_object);
pub struct RcMyTrait;
derive_rc_object!(RcMyTrait, MyTrait, bin_object, json_object);
pub struct RcWeakMyTrait;
derive_rc_weak_object!(RcWeakMyTrait, MyTrait, bin_object, json_object);

pub struct ArcMyTrait;
derive_arc_object!(ArcMyTrait, MyTrait, bin_object, json_object);

pub trait MyTrait:
    'static
    + Debug
    + RawAny
    + AsDiscriminant<BoxMyTrait>
    + AsDiscriminant<RcMyTrait>
    + AsDiscriminant<ArcMyTrait>
    + AsDiscriminant<RcWeakMyTrait>
{
}

impl MyTrait for A {}
impl MyTrait for B {}

#[derive(Serialize, Deserialize, Debug, Eq, PartialEq, Ord, PartialOrd)]
struct A(u8);

#[derive(Serialize, Deserialize, Debug, Eq, PartialEq, Ord, PartialOrd)]
struct B(u16);

derive_variant!(RcMyTrait, A);
derive_variant!(RcMyTrait, B);
derive_variant!(RcWeakMyTrait, A);
derive_variant!(RcWeakMyTrait, B);
derive_variant!(BoxMyTrait, A);
derive_variant!(BoxMyTrait, B);
derive_variant!(ArcMyTrait, A);
derive_variant!(ArcMyTrait, B);

derive_deserialize_rc_shared!(A);
derive_deserialize_rc_shared!(B);
derive_deserialize_rc_weak_shared!(A);
derive_deserialize_rc_weak_shared!(B);
derive_deserialize_arc_shared!(A);
derive_deserialize_arc_shared!(B);
derive_deserialize_arc_weak_shared!(A);
derive_deserialize_arc_weak_shared!(B);
derive_serialize_rc_shared!(A);
derive_serialize_rc_shared!(B);
derive_serialize_arc_shared!(A);
derive_serialize_arc_shared!(B);
derive_serialize_rc_weak_shared!(A);
derive_serialize_rc_weak_shared!(B);
derive_serialize_arc_weak_shared!(A);
derive_serialize_arc_weak_shared!(B);

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
    assert!(
        expected.contains(&compare),
        "{:?} \n{:?}",
        compare,
        expected
    );
    let output = BinDecoderBuilder::new(&found, &mut BinDecoderSchema::new())
        .deserialize::<T>(&mut Context::new())?;
    Ok(output)
}

#[test]
fn test_json_rc() -> anyhow::Result<()> {
    let input = Rc::new(A(42u8)) as Rc<dyn MyTrait>;
    let output = json_round_trip(
        &input,
        r#"{
  "test::A": [
    {
      "id": 0,
      "inner": [
        42
      ]
    }
  ]
}"#,
    )?;
    let output: &A = &*Rc::<dyn Any>::downcast::<A>(output).unwrap();
    assert_eq!(output, &A(42));
    Ok(())
}

#[test]
fn test_json_box() -> anyhow::Result<()> {
    let input = Box::new(A(42u8)) as Box<dyn MyTrait>;
    let output = json_round_trip(
        &input,
        r#"{
  "test::A": [
    [
      42
    ]
  ]
}"#,
    )?;
    let output: &A = &*Box::<dyn Any>::downcast::<A>(output).unwrap();
    assert_eq!(output, &A(42));
    Ok(())
}

#[test]
fn test_bin() -> anyhow::Result<()> {
    let input = Rc::new(A(42u8)) as Rc<dyn MyTrait>;
    let output = bin_round_trip(
        &input,
        &[
            &[
                21, 2, //
                7, b't', b'e', b's', b't', b':', b':', b'A', //
                7, b't', b'e', b's', b't', b':', b':', b'B', //
                18, 0, 0, //
                17, 1, //
                21, 2, 2, b'i', b'd', 5, b'i', b'n', b'n', b'e', b'r', //
                16, 1, 10, 0, 26, 17, 1, 7, 42,
            ],
            &[
                21, 2, //
                7, b't', b'e', b's', b't', b':', b':', b'B', //
                7, b't', b'e', b's', b't', b':', b':', b'A', //
                18, 0, 1, //
                17, 1, //
                21, 2, 2, b'i', b'd', 5, b'i', b'n', b'n', b'e', b'r', //
                16, 1, 10, 0, 26, 17, 1, 7, 42,
            ],
            //
        ],
    )?;
    let output: &A = &*Rc::<dyn Any>::downcast::<A>(output).unwrap();
    assert_eq!(output, &A(42));
    Ok(())
}
