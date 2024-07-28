use marshal_core::decode::{AnyDecoder, DecodeHint, Decoder};

use crate::context::Context;
use marshal_core::encode::{AnyEncoder, Encoder};

use crate::ser::Serialize;

use crate::de::Deserialize;

impl<D: Decoder> Deserialize<D> for String {
    fn deserialize<'p, 'de>(d: AnyDecoder<'p, 'de, D>, _ctx: Context) -> anyhow::Result<Self> {
        Ok(d.decode(DecodeHint::String)?
            .try_into_string()?
            .into_owned())
    }
}

impl<W: Encoder> Serialize<W> for String {
    fn serialize<'w, 'en>(&self, w: AnyEncoder<'w, 'en, W>, _ctx: Context) -> anyhow::Result<()> {
        w.encode_str(self)
    }
}
