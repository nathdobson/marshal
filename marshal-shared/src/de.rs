use marshal::context::Context;
use marshal::de::Deserialize;
use marshal::decode::{AnyDecoder, DecodeHint, Decoder, DecoderView};
use marshal::encode::Encoder;
use marshal::reexports::anyhow;
use marshal::ser::Serialize;
use marshal::Deserialize;
use std::any::Any;
use std::collections::HashMap;
use std::rc::Rc;
use weak_table::ptr_weak_key_hash_map::Entry;

pub struct SharedDeserializeContext {
    refs: HashMap<usize, Rc<dyn Any>>,
}

#[derive(Deserialize)]
enum Shared<T> {
    Value(T),
    Reference(usize),
}

fn deserialize_shared<'de, D: Decoder<'de>, T: 'static + Deserialize<'de, D>>(
    d: D::AnyDecoder<'_>,
    ctx: &mut Context,
) -> anyhow::Result<Rc<T>> {
    match <Shared<T> as Deserialize<'de, D>>::deserialize(d, ctx)? {
        Shared::Value(_) => todo!(),
        Shared::Reference(_) => todo!(),
    }
}
