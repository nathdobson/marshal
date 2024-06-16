use std::rc::Rc;

use marshal::{Deserialize, Serialize};
use marshal::context::Context;
use marshal_json::decode::full::JsonDecoderBuilder;
use marshal_json::encode::full::JsonEncoderBuilder;
use marshal_shared::{derive_deserialize_rc_shared, derive_serialize_rc_shared};

#[derive(Serialize, Deserialize)]
struct Foo(u8);

derive_deserialize_rc_shared!(Foo);
derive_serialize_rc_shared!(Foo);

#[test]
fn test() -> anyhow::Result<()> {
    let rc1 = Rc::new(Foo(4));
    let rc2 = Rc::new(Foo(8));
    let list = vec![rc1.clone(), rc1.clone(), rc2];
    let encoded = JsonEncoderBuilder::new().serialize(&list, &mut Context::new())?;
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
  }
]"#
    );
    let decoded = JsonDecoderBuilder::new(encoded.as_bytes())
        .deserialize::<Vec<Rc<Foo>>>(&mut Context::new())?;
    assert_eq!(decoded[0].0, 4);
    assert_eq!(decoded[1].0, 4);
    assert_eq!(decoded[2].0, 8);
    assert!(Rc::ptr_eq(&decoded[0], &decoded[1]));
    Ok(())
}
