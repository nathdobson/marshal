use marshal_core::decode::{AnyDecoder, Decoder};

use crate::context::Context;
use crate::de::{Deserialize, SchemaError};

impl<D: Decoder> Deserialize<D> for ! {
    fn deserialize<'p, 'de>(_: AnyDecoder<'p, 'de, D>, _ctx: Context) -> anyhow::Result<Self> {
        Err(SchemaError::UninhabitedType.into())
    }
}
