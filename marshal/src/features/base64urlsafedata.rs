use base64urlsafedata::{Base64UrlSafeData, HumanBinaryData};

use marshal_core::decode::{AnyDecoder, Decoder};
use marshal_core::encode::{AnyEncoder, Encoder};

use crate::context::Context;
use crate::de::Deserialize;
use crate::ser::Serialize;

impl<E: Encoder> Serialize<E> for HumanBinaryData {
    fn serialize<'w, 'en>(&self, e: AnyEncoder<'w, 'en, E>, _ctx: Context) -> anyhow::Result<()> {
        e.encode_bytes(self)
    }
}

impl<E: Encoder> Serialize<E> for Base64UrlSafeData {
    fn serialize<'w, 'en>(&self, e: AnyEncoder<'w, 'en, E>, _ctx: Context) -> anyhow::Result<()> {
        e.encode_bytes(self)
    }
}

impl<D: Decoder> Deserialize<D> for HumanBinaryData {
    fn deserialize<'p, 'de>(d: AnyDecoder<'p, 'de, D>, ctx: Context) -> anyhow::Result<Self> {
        Ok(HumanBinaryData::from(
            <Vec<u8> as Deserialize<D>>::deserialize(d, ctx)?,
        ))
    }
}

impl<D: Decoder> Deserialize<D> for Base64UrlSafeData {
    fn deserialize<'p, 'de>(d: AnyDecoder<'p, 'de, D>, ctx: Context) -> anyhow::Result<Self> {
        Ok(Base64UrlSafeData::from(
            <Vec<u8> as Deserialize<D>>::deserialize(d, ctx)?,
        ))
    }
}
