use marshal::context::Context;
use marshal::encode::Encoder;
use marshal::reexports::anyhow;
use marshal::ser::Serialize;
use marshal::Serialize;
use std::any::Any;
use std::rc;
use std::rc::Rc;
use weak_table::ptr_weak_key_hash_map::Entry;
use weak_table::PtrWeakKeyHashMap;

#[derive(Default)]
pub struct SharedSerializeContext {
    next_id: usize,
    ids: PtrWeakKeyHashMap<rc::Weak<dyn Any>, usize>,
}

#[derive(Serialize)]
struct Shared<'a, T> {
    id: usize,
    value: Option<&'a T>,
}

fn serialize_shared<E: Encoder, T: 'static + Serialize<E>>(
    ptr: &Rc<T>,
    e: E::AnyEncoder<'_>,
    ctx: &mut Context,
) -> anyhow::Result<()> {
    let shared_ctx = ctx.get_or_default::<SharedSerializeContext>();
    match shared_ctx.ids.entry(ptr.clone()) {
        Entry::Occupied(entry) => {
            Shared::<T> {
                id: *entry.get(),
                value: None,
            }
            .serialize(e, ctx)?;
            Ok(())
        }
        Entry::Vacant(entry) => {
            let id = shared_ctx.next_id;
            entry.insert(id);
            shared_ctx.next_id += 1;
            Shared {
                id,
                value: Some(&**ptr),
            }
            .serialize(e, ctx)?;
            Ok(())
        }
    }
}
