use std::any::Any;
use std::rc;
use std::rc::Rc;

use pretty_assertions::assert_eq;

use marshal::context::OwnedContext;
use marshal::{Deserialize, Serialize};
use marshal_json::decode::full::JsonDecoderBuilder;
use marshal_json::encode::full::JsonEncoderBuilder;
use marshal_pointer::{Rcf, RcfWeak};
use marshal_pointer::raw_any::DerefRaw;
use marshal_shared::de::SharedRcDeserializeContext;
use marshal_shared::ser::SharedSerializeContext;
use marshal_shared::{
    derive_deserialize_rc_shared, derive_deserialize_rc_weak_shared, derive_serialize_rc_shared,
    derive_serialize_rc_weak_shared,
};

#[derive(Serialize, Deserialize)]
struct Foo(u8);

derive_deserialize_rc_shared!(Foo);
derive_serialize_rc_shared!(Foo);
derive_deserialize_rc_weak_shared!(Foo);
derive_serialize_rc_weak_shared!(Foo);

#[test]
fn test() -> anyhow::Result<()> {
    let rc1 = Rcf::new(Foo(4));
    let rc2 = Rcf::new(Foo(8));
    let rc3 = Rcf::new(Foo(15));
    type Tuple = (Rcf<Foo>, Rcf<Foo>, Rcf<Foo>, RcfWeak<Foo>, Rcf<Foo>);
    let list: Tuple = (rc1.clone(), rc1.clone(), rc2, Rcf::downgrade(&rc3), rc3);
    let mut ser_ctx = OwnedContext::new();
    let mut shared_ser_ctx = SharedSerializeContext::<RcfWeak<dyn Any>>::default();
    ser_ctx.insert_mut(&mut shared_ser_ctx);
    let encoded = JsonEncoderBuilder::new().serialize(&list, ser_ctx.borrow())?;
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
    let mut de_ctx = OwnedContext::new();
    let mut shared_de_ctx = SharedRcDeserializeContext::default();
    de_ctx.insert_mut(&mut shared_de_ctx);

    let decoded =
        JsonDecoderBuilder::new(encoded.as_bytes()).deserialize::<Tuple>(de_ctx.borrow())?;
    assert_eq!(decoded.0 .0, 4);
    assert_eq!(decoded.1 .0, 4);
    assert_eq!(decoded.2 .0, 8);
    assert_eq!(decoded.3.upgrade().unwrap().0, 15);
    assert_eq!(decoded.4 .0, 15);
    assert_eq!(decoded.0.deref_raw(), decoded.1.deref_raw());
    assert_eq!(decoded.3.upgrade().unwrap().deref_raw(), decoded.4.deref_raw());
    Ok(())
}
