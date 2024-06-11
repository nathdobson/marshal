use marshal::context::Context;
use marshal::encode::{AnyEncoder, Encoder, TupleVariantEncoder};
use marshal::ser::Serialize;

use crate::Object;

pub fn serialize_object<E: Encoder, T: ?Sized + Object + Serialize<E>>(
    value: &T,
    e: E::AnyEncoder<'_>,
    ctx: &mut Context,
) -> anyhow::Result<()> {
    let mut e = e.encode_tuple_variant(
        T::object_descriptor().object_name(),
        T::object_descriptor().discriminant_names(),
        value.as_discriminant(),
        1,
    )?;
    <T as Serialize<E>>::serialize(value, e.encode_field()?, ctx)?;
    e.end()?;
    Ok(())
}