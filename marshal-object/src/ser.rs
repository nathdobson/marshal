use marshal::context::Context;
use marshal::encode::{AnyEncoder, Encoder, SomeEncoder, TupleVariantEncoder};
use marshal::ser::Serialize;
use std::rc;

use crate::{AsDiscriminant, Object};

pub fn serialize_object<O: Object, E: Encoder>(
    value: &O::Dyn,
    e: E::AnyEncoder<'_>,
    ctx: &mut Context,
) -> anyhow::Result<()>
where
    O::Dyn: Serialize<E>,
{
    let mut e = e.encode_tuple_variant(
        O::object_descriptor().object_name(),
        O::object_descriptor().discriminant_names(),
        (value as *const O::Dyn).as_discriminant(),
        1,
    )?;
    <O::Dyn as Serialize<E>>::serialize(value, e.encode_field()?, ctx)?;
    e.end()?;
    Ok(())
}

pub fn serialize_rc_weak_object<O: Object, E: Encoder>(
    value: &rc::Weak<O::Dyn>,
    e: E::AnyEncoder<'_>,
    ctx: &mut Context,
) -> anyhow::Result<()>
where
    O::Dyn: Serialize<E>,
{
    let mut e = e.encode_tuple_variant(
        O::object_descriptor().object_name(),
        O::object_descriptor().discriminant_names(),
        value.as_ptr().as_discriminant(),
        1,
    )?;
    if let Some(value) = value.upgrade() {
        let mut e = e.encode_field()?.encode_some()?;
        <O::Dyn as Serialize<E>>::serialize(&value, e.encode_some()?, ctx)?;
        e.end()?;
    } else {
        e.encode_field()?.encode_none()?;
    }
    e.end()?;
    Ok(())
}
