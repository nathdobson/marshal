use marshal_core::decode::{AnyGenDecoder, GenDecoder};

use crate::context::Context;
use crate::de::{Deserialize, SchemaError};

impl<D: GenDecoder> Deserialize<D> for ! {
    fn deserialize<'p, 'de>(_: AnyGenDecoder<'p, 'de, D>, _ctx: Context) -> anyhow::Result<Self> {
        Err(SchemaError::UninhabitedType.into())
    }
}
