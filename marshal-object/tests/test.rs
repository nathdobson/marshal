#![feature(trait_alias)]
#![feature(trait_upcasting)]
#![feature(const_trait_impl)]
#![feature(unsize)]
#![feature(coerce_unsized)]
#![feature(arbitrary_self_types)]

use std::any::Any;
use std::fmt::Debug;

use marshal::context::OwnedContext;
use marshal_bin::decode::BinDecoderSchema;
use marshal_bin::decode::full::BinDecoderBuilder;
use marshal_bin::DeserializeBin;
use marshal_bin::encode::BinEncoderSchema;
use marshal_bin::encode::full::BinEncoderBuilder;
use marshal_bin::SerializeBin;
use marshal_bin::VU128_MAX_PADDING;
use marshal_json::decode::full::JsonDecoderBuilder;
use marshal_json::DeserializeJson;
use marshal_json::encode::full::JsonEncoderBuilder;
use marshal_json::SerializeJson;
use marshal_pointer::{Rcf, RcfWeak};
use marshal_pointer::raw_any::RawAny;
use marshal_shared::de::SharedRcDeserializeContext;
use marshal_shared::ser::SharedSerializeContext;

use crate::x::{A, MyTrait};

#[no_implicit_prelude]
mod x {
    pub struct BoxMyTrait;
    ::marshal_object::derive_box_object!(BoxMyTrait, MyTrait);
    ::marshal_object::derive_serialize_provider!(
        BoxMyTrait,
        ::marshal_bin::encode::full::BinEncoder,
        ::marshal_json::encode::full::JsonEncoder
    );
    ::marshal_object::derive_deserialize_provider!(
        BoxMyTrait,
        ::marshal_bin::decode::full::BinDecoder,
        ::marshal_json::decode::full::JsonDecoder
    );
    pub struct RcMyTrait;
    ::marshal_object::derive_rc_object!(RcMyTrait, MyTrait);
    ::marshal_object::derive_serialize_provider!(
        RcMyTrait,
        ::marshal_bin::encode::full::BinEncoder,
        ::marshal_json::encode::full::JsonEncoder
    );
    ::marshal_object::derive_deserialize_provider!(
        RcMyTrait,
        ::marshal_bin::decode::full::BinDecoder,
        ::marshal_json::decode::full::JsonDecoder
    );
    pub struct RcWeakMyTrait;
    ::marshal_object::derive_rc_weak_object!(RcWeakMyTrait, MyTrait);
    ::marshal_object::derive_serialize_provider!(
        RcWeakMyTrait,
        ::marshal_bin::encode::full::BinEncoder,
        ::marshal_json::encode::full::JsonEncoder
    );
    ::marshal_object::derive_deserialize_provider!(
        RcWeakMyTrait,
        ::marshal_bin::decode::full::BinDecoder,
        ::marshal_json::decode::full::JsonDecoder
    );

    pub struct ArcMyTrait;
    ::marshal_object::derive_arc_object!(ArcMyTrait, MyTrait);
    ::marshal_object::derive_serialize_provider!(
        ArcMyTrait,
        ::marshal_bin::encode::full::BinEncoder,
        ::marshal_json::encode::full::JsonEncoder
    );
    ::marshal_object::derive_deserialize_provider!(
        ArcMyTrait,
        ::marshal_bin::decode::full::BinDecoder,
        ::marshal_json::decode::full::JsonDecoder
    );

    pub struct ArcWeakMyTrait;
    ::marshal_object::derive_arc_weak_object!(ArcWeakMyTrait, MyTrait);
    ::marshal_object::derive_serialize_provider!(
        ArcWeakMyTrait,
        ::marshal_bin::encode::full::BinEncoder,
        ::marshal_json::encode::full::JsonEncoder
    );
    ::marshal_object::derive_deserialize_provider!(
        ArcWeakMyTrait,
        ::marshal_bin::decode::full::BinDecoder,
        ::marshal_json::decode::full::JsonDecoder
    );

    pub trait MyTrait:
        'static
        + ::std::fmt::Debug
        + ::marshal_pointer::raw_any::RawAny
        + ::marshal_object::AsDiscriminant<BoxMyTrait>
        + ::marshal_object::AsDiscriminant<RcMyTrait>
        + ::marshal_object::AsDiscriminant<ArcMyTrait>
        + ::marshal_object::AsDiscriminant<RcWeakMyTrait>
        + ::marshal_object::AsDiscriminant<ArcWeakMyTrait>
    {
    }

    impl MyTrait for A {}
    impl MyTrait for B {}

    #[derive(
        ::marshal::Serialize, ::marshal::Deserialize, Debug, Eq, PartialEq, Ord, PartialOrd,
    )]
    pub struct A(pub u8);

    #[derive(
        ::marshal::Serialize, ::marshal::Deserialize, Debug, Eq, PartialEq, Ord, PartialOrd,
    )]
    pub struct B(pub u16);

    ::marshal_object::derive_variant!(RcMyTrait, A);
    ::marshal_object::derive_variant!(RcMyTrait, B);
    ::marshal_object::derive_variant!(RcWeakMyTrait, A);
    ::marshal_object::derive_variant!(RcWeakMyTrait, B);
    ::marshal_object::derive_variant!(BoxMyTrait, A);
    ::marshal_object::derive_variant!(BoxMyTrait, B);
    ::marshal_object::derive_variant!(ArcMyTrait, A);
    ::marshal_object::derive_variant!(ArcMyTrait, B);
    ::marshal_object::derive_variant!(ArcWeakMyTrait, A);
    ::marshal_object::derive_variant!(ArcWeakMyTrait, B);

    ::marshal_shared::derive_deserialize_rc_shared!(A);
    ::marshal_shared::derive_deserialize_rc_shared!(B);
    ::marshal_shared::derive_deserialize_rc_weak_shared!(A);
    ::marshal_shared::derive_deserialize_rc_weak_shared!(B);
    ::marshal_shared::derive_deserialize_arc_shared!(A);
    ::marshal_shared::derive_deserialize_arc_shared!(B);
    ::marshal_shared::derive_deserialize_arc_weak_shared!(A);
    ::marshal_shared::derive_deserialize_arc_weak_shared!(B);
    ::marshal_shared::derive_serialize_rc_shared!(A);
    ::marshal_shared::derive_serialize_rc_shared!(B);
    ::marshal_shared::derive_serialize_arc_shared!(A);
    ::marshal_shared::derive_serialize_arc_shared!(B);
    ::marshal_shared::derive_serialize_rc_weak_shared!(A);
    ::marshal_shared::derive_serialize_rc_weak_shared!(B);
    ::marshal_shared::derive_serialize_arc_weak_shared!(A);
    ::marshal_shared::derive_serialize_arc_weak_shared!(B);
}

#[track_caller]
pub fn json_round_trip<T: Debug + SerializeJson + DeserializeJson>(
    input: &T,
    expected: &str,
) -> anyhow::Result<T> {
    let mut ser_ctx = OwnedContext::new();
    let mut shared_ser_ctx = SharedSerializeContext::<RcfWeak<dyn Any>>::default();
    ser_ctx.insert_mut(&mut shared_ser_ctx);
    println!("{:?}", input);
    let found = JsonEncoderBuilder::new().serialize(input, ser_ctx.borrow())?;
    assert_eq!(expected.trim_start(), found);
    let mut de_ctx = OwnedContext::new();
    let mut shared_de_ctx = SharedRcDeserializeContext::default();
    de_ctx.insert_mut(&mut shared_de_ctx);
    let output = JsonDecoderBuilder::new(found.as_bytes()).deserialize::<T>(de_ctx.borrow())?;
    Ok(output)
}

#[track_caller]
pub fn bin_round_trip<T: Debug + SerializeBin + DeserializeBin>(
    input: &T,
    expected: &[&[u8]],
) -> anyhow::Result<T> {
    let mut ser_ctx = OwnedContext::new();
    let mut shared_ser_ctx = SharedSerializeContext::<RcfWeak<dyn Any>>::default();
    ser_ctx.insert_mut(&mut shared_ser_ctx);
    println!("{:?}", input);
    let found =
        BinEncoderBuilder::new(&mut BinEncoderSchema::new()).serialize(input, ser_ctx.borrow())?;
    let compare = &found[0..found.len() - VU128_MAX_PADDING];
    assert!(
        expected.contains(&compare),
        "{:?} \n{:?}",
        compare,
        expected
    );
    let mut de_ctx = OwnedContext::new();
    let mut shared_de_ctx = SharedRcDeserializeContext::default();
    de_ctx.insert_mut(&mut shared_de_ctx);
    let output = BinDecoderBuilder::new(&found, &mut BinDecoderSchema::new())
        .deserialize::<T>(de_ctx.borrow())?;
    Ok(output)
}

#[test]
fn test_json_rc() -> anyhow::Result<()> {
    let input = Rcf::new(A(42u8)) as Rcf<dyn MyTrait>;
    let output = json_round_trip(
        &input,
        r#"{
  "test::x::A": [
    {
      "id": 0,
      "inner": [
        42
      ]
    }
  ]
}"#,
    )?;
    let output: &A = &*Rcf::<dyn RawAny>::downcast::<A>(output).unwrap();
    assert_eq!(output, &A(42));
    Ok(())
}

#[test]
fn test_json_box() -> anyhow::Result<()> {
    let input = Box::new(A(42u8)) as Box<dyn MyTrait>;
    let output = json_round_trip(
        &input,
        r#"{
  "test::x::A": [
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
    let input = Rcf::new(A(42u8)) as Rcf<dyn MyTrait>;
    let output = bin_round_trip(
        &input,
        &[
            &[
                21, 2, //
                10, b't', b'e', b's', b't', b':', b':', b'x', b':', b':', b'A', //
                10, b't', b'e', b's', b't', b':', b':', b'x', b':', b':', b'B', //
                18, 0, 0, //
                17, 1, //
                21, 2, 2, b'i', b'd', 5, b'i', b'n', b'n', b'e', b'r', //
                16, 1, 10, 0, 26, 17, 1, 7, 42,
            ],
            &[
                21, 2, //
                10, b't', b'e', b's', b't', b':', b':', b'x', b':', b':', b'B', //
                10, b't', b'e', b's', b't', b':', b':', b'x', b':', b':', b'A', //
                18, 0, 1, //
                17, 1, //
                21, 2, 2, b'i', b'd', 5, b'i', b'n', b'n', b'e', b'r', //
                16, 1, 10, 0, 26, 17, 1, 7, 42,
            ],
            //
        ],
    )?;
    let output: &A = &*Rcf::<dyn RawAny>::downcast::<A>(output).unwrap();
    assert_eq!(output, &A(42));
    Ok(())
}
