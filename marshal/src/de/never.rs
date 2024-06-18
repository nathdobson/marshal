use marshal_core::decode::{AnyDecoder, Decoder};

use crate::context::Context;
use crate::de::{Deserialize, SchemaError};

impl<'de, P: Decoder<'de>> Deserialize<'de, P> for ! {
    fn deserialize<'p>(_: AnyDecoder<'p, 'de, P>, _ctx: &mut Context) -> anyhow::Result<Self> {
        Err(SchemaError::UninhabitedType.into())
    }
}
