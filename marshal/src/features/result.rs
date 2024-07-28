use marshal_core::decode::{AnyDecoder, Decoder};

use crate::context::Context;
use crate::de::Deserialize;

use marshal_core::encode::{AnyEncoder, Encoder};

use crate::ser::Serialize;

impl<D: Decoder, A: Deserialize<D>, B: Deserialize<D>> Deserialize<D> for Result<A, B> {
    fn deserialize<'p, 'de>(d: AnyDecoder<'p, 'de, D>, ctx: Context) -> anyhow::Result<Self> {
        let (disc, mut variant) = d.decode_enum_helper("Result", &["Ok", "Err"])?;
        let result = match disc {
            0 => {
                let mut tuple = variant.decode_tuple_variant_helper(1)?;
                let value = A::deserialize(tuple.decode_next()?, ctx)?;
                tuple.decode_end(1)?;
                Ok(value)
            }
            1 => {
                let mut tuple = variant.decode_tuple_variant_helper(1)?;
                let value = B::deserialize(tuple.decode_next()?, ctx)?;
                tuple.decode_end(1)?;
                Err(value)
            }
            _ => unreachable!(),
        };
        variant.decode_end()?;
        Ok(result)
    }
}

impl<E: Encoder, A: Serialize<E>, B: Serialize<E>> Serialize<E> for Result<A, B> {
    fn serialize<'w, 'en>(&self, e: AnyEncoder<'w, 'en, E>, ctx: Context) -> anyhow::Result<()> {
        let name = "Result";
        let variants = &["Ok", "Err"];
        match self {
            Ok(a) => {
                let mut e = e.encode_tuple_variant(name, variants, 0, 1)?;
                a.serialize(e.encode_field()?, ctx)?;
                e.end()?;
                Ok(())
            }
            Err(b) => {
                let mut e = e.encode_tuple_variant(name, variants, 1, 1)?;
                b.serialize(e.encode_field()?, ctx)?;
                e.end()?;
                Ok(())
            }
        }
    }
}