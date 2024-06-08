use crate::context::Context;
use crate::de::{Deserialize, SchemaError};
use marshal_core::decode::Parser;

impl<'de, P: Parser<'de>> Deserialize<'de,P> for ! {
    fn deserialize<'p>(_: P::AnyParser<'p>, _ctx: &mut Context) -> anyhow::Result<Self> {
        Err(SchemaError::Never.into())
    }
}
