use marshal::context::Context;
use marshal::encode::{AnyEncoder, Encoder, TupleVariantEncoder};
use marshal::ser::Serialize;

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
        value.as_discriminant(),
        1,
    )?;
    <O::Dyn as Serialize<E>>::serialize(value, e.encode_field()?, ctx)?;
    e.end()?;
    Ok(())
}
