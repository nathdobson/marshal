use marshal_core::decode::{AnyDecoder, Decoder};

use crate::context::Context;
use crate::de::Deserialize;

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
