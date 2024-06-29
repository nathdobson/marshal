use marshal_core::decode::{AnyDecoder, Decoder};

use crate::context::Context;
use crate::de::{Deserialize, SchemaError};

impl<P: Decoder> Deserialize<P> for ! {
    fn deserialize<'p>(_: AnyDecoder<'p, P>, _ctx: Context) -> anyhow::Result<Self> {
        Err(SchemaError::UninhabitedType.into())
    }
}
