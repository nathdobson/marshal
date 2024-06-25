use std::any::Any;
use std::rc;
use std::rc::Rc;

use pretty_assertions::assert_eq;

use marshal::context::Context;
use marshal::{Deserialize, Serialize};
use marshal_json::decode::full::JsonDecoderBuilder;
use marshal_json::encode::full::JsonEncoderBuilder;
use marshal_shared::ser::SharedSerializeContext;
use marshal_shared::{
    derive_deserialize_rc_shared, derive_deserialize_rc_weak_shared, derive_serialize_rc_shared,
    derive_serialize_rc_weak_shared,
};
use marshal_shared::de::SharedRcDeserializeContext;

#[derive(Serialize, Deserialize)]
struct Foo(u8);

derive_deserialize_rc_shared!(Foo);
derive_serialize_rc_shared!(Foo);
derive_deserialize_rc_weak_shared!(Foo);
derive_serialize_rc_weak_shared!(Foo);

#[test]
fn test() -> anyhow::Result<()> {
    let rc1 = Rc::new(Foo(4));
    let rc2 = Rc::new(Foo(8));
    let rc3 = Rc::new(Foo(15));
    type Tuple = (Rc<Foo>, Rc<Foo>, Rc<Foo>, rc::Weak<Foo>, Rc<Foo>);
    let list: Tuple = (rc1.clone(), rc1.clone(), rc2, Rc::downgrade(&rc3), rc3);
    let mut ser_ctx = Context::new();
    let mut shared_ser_ctx = SharedSerializeContext::<rc::Weak<dyn Any>>::default();
    ser_ctx.insert(&mut shared_ser_ctx);
    let encoded = JsonEncoderBuilder::new().serialize(&list, &mut ser_ctx)?;
    assert_eq!(
        encoded,
        r#"[
  {
    "id": 0,
    "inner": [
      4
    ]
  },
  {
    "id": 0,
    "inner": null
  },
  {
    "id": 1,
    "inner": [
      8
    ]
  },
  2,
  {
    "id": 2,
    "inner": [
      15
    ]
  }
]"#
    );
    let mut de_ctx = Context::new();
    let mut shared_de_ctx = SharedRcDeserializeContext::default();
    de_ctx.insert(&mut shared_de_ctx);

    let decoded =
        JsonDecoderBuilder::new(encoded.as_bytes()).deserialize::<Tuple>(&mut de_ctx)?;
    assert_eq!(decoded.0 .0, 4);
    assert_eq!(decoded.1 .0, 4);
    assert_eq!(decoded.2 .0, 8);
    assert_eq!(decoded.3.upgrade().unwrap().0, 15);
    assert_eq!(decoded.4 .0, 15);
    assert!(Rc::ptr_eq(&decoded.0, &decoded.1));
    assert!(Rc::ptr_eq(&decoded.3.upgrade().unwrap(), &decoded.4));
    Ok(())
}
