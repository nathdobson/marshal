use marshal_core::decode::{AnyDecoder, Decoder};
use marshal_core::SchemaError;

use crate::context::Context;
use crate::de::Deserialize;
use marshal_core::encode::{AnyEncoder, Encoder};

use crate::ser::Serialize;

impl<D: Decoder> Deserialize<D> for ! {
    fn deserialize<'p, 'de>(_: AnyDecoder<'p, 'de, D>, _ctx: Context) -> anyhow::Result<Self> {
        Err(SchemaError::UninhabitedType.into())
    }
}

impl<W: Encoder> Serialize<W> for ! {
    fn serialize<'w, 'en>(&self, _: AnyEncoder<'w, 'en, W>, _ctx: Context) -> anyhow::Result<()> {
        *self
    }
}
