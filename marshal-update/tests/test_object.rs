#![deny(unused_must_use)]
#![feature(arbitrary_self_types)]
#![feature(unsize)]

use marshal::context::Context;
use marshal::de::Deserialize;
use marshal::decode::AnyDecoder;
use marshal::encode::AnyEncoder;
use marshal_bin::decode::full::BinDecoder;
use marshal_bin::encode::full::BinEncoder;
use std::any::Any;
use std::fmt::Debug;
// use marshal_bin::bin_object;
use marshal_derive::{Deserialize, DeserializeUpdate, Serialize, SerializeStream, SerializeUpdate};
use marshal_json::decode::full::JsonDecoder;
use marshal_json::encode::full::JsonEncoder;
use marshal_object::{
    derive_box_object, derive_deserialize_provider, derive_serialize_provider, derive_variant,
    AsDiscriminant,
};
use marshal_pointer::RawAny;
use marshal_update::de::DeserializeUpdate;
use marshal_update::object_map::ObjectMap;
use marshal_update::ser::{
     SerializeStream, SerializeStreamDyn, SerializeUpdate, SerializeUpdateDyn,
};
use marshal_update::tester::Tester;

pub struct BoxFoo;
derive_box_object!(BoxFoo, Foo);
derive_serialize_provider!(BoxFoo, BinEncoder, JsonEncoder);
derive_deserialize_provider!(BoxFoo, BinDecoder, JsonDecoder);
pub trait Foo:
    'static
    + Debug
    + RawAny
    + AsDiscriminant<BoxFoo>
    + SerializeUpdateDyn<JsonEncoder>
    + DeserializeUpdate<JsonDecoder>
{
}

impl SerializeStream for Box<dyn Foo> {
    type Stream = Box<dyn Sync + Send + RawAny>;
    fn start_stream(&self, ctx: Context) -> anyhow::Result<Self::Stream> {
        Ok((**self).start_stream_dyn(ctx)?)
    }
}

impl SerializeUpdate<JsonEncoder> for Box<dyn Foo> {
    fn serialize_update(
        &self,
        stream: &mut Self::Stream,
        e: AnyEncoder<JsonEncoder>,
        ctx: Context,
    ) -> anyhow::Result<()> {
        (**self).serialize_update_dyn(stream, e, ctx)
    }
}

impl DeserializeUpdate<JsonDecoder> for Box<dyn Foo> {
    fn deserialize_update<'p, 'de>(
        &mut self,
        d: AnyDecoder<'p, 'de, JsonDecoder>,
        ctx: Context,
    ) -> anyhow::Result<()> {
        (**self).deserialize_update(d, ctx)
    }
}

derive_variant!(BoxFoo, A);
#[derive(
    Serialize,
    SerializeUpdate,
    SerializeStream,
    DeserializeUpdate,
    Deserialize,
    Debug,
    Eq,
    PartialEq,
    Ord,
    PartialOrd,
)]
struct A(u8);
impl Foo for A {}

derive_variant!(BoxFoo, B);
#[derive(
    Serialize,
    SerializeUpdate,
    SerializeStream,
    DeserializeUpdate,
    Deserialize,
    Debug,
    Eq,
    PartialEq,
    Ord,
    PartialOrd,
)]
struct B(u16);
impl Foo for B {}

#[test]
fn test() -> anyhow::Result<()> {
    let mut map = ObjectMap::<BoxFoo>::new();
    map.insert(Box::new(A(4)));
    let mut tester = Tester::new(
        map,
        r#"{
  "test_object::A": {
    "test_object::A": [
      [
        4
      ]
    ]
  }
}"#,
    )?;
    assert_eq!(tester.output().get::<A>().unwrap().0, 4);
    tester.next("{}")?;
    tester.input_mut().insert(Box::new(B(8)));
    tester.next(
        r#"{
  "test_object::B": {
    "test_object::B": [
      [
        8
      ]
    ]
  }
}"#,
    )?;
    assert_eq!(tester.output().get::<B>().unwrap().0, 8);
    tester.input_mut().get_mut::<B>().unwrap().0 = 15;
    tester.next(
        r#"{
  "test_object::B": [
    15
  ]
}"#,
    )?;
    Ok(())
}
