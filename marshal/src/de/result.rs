use marshal_core::decode::{AnyDecoder, Decoder};

use crate::context::Context;
use crate::de::Deserialize;

impl<D: Decoder, A: Deserialize<D>, B: Deserialize<D>> Deserialize<D> for Result<A, B> {
    fn deserialize<'p, 'de>(d: AnyDecoder<'p, 'de, D>, ctx: Context) -> anyhow::Result<Self> {
        let (disc, mut variant) = d.decode_enum_helper("Result", &["Ok", "Err"])?;
        match disc {
            0 => {
                let mut variant = variant.decode_tuple_variant_helper(1)?;
                let value = A::deserialize(variant.decode_next()?, ctx)?;
                variant.decode_end()?;
                Ok(Ok(value))
            }
            1 => {
                let mut variant = variant.decode_tuple_variant_helper(1)?;
                let value = B::deserialize(variant.decode_next()?, ctx)?;
                variant.decode_end()?;
                Ok(Err(value))
            }
            _ => unreachable!(),
        }
    }
}
