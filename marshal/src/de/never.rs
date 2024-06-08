use crate::context::Context;
use crate::de::{Deserialize, SchemaError};
use marshal_core::decode::Decoder;

impl<'de, P: Decoder<'de>> Deserialize<'de, P> for ! {
    fn deserialize<'p>(_: P::AnyDecoder<'p>, _ctx: &mut Context) -> anyhow::Result<Self> {
        Err(SchemaError::UninhabitedType.into())
    }
}
