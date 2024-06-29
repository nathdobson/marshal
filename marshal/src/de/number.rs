use marshal_core::decode::{AnyDecoder, DecodeHint, Decoder, DecoderView};
use marshal_core::{Primitive, PrimitiveType};

use crate::context::Context;
use crate::de::Deserialize;

macro_rules! derive_number {
    ($t:ty, $v:ident) => {
        impl<P: Decoder> Deserialize<P> for $t {
            fn deserialize<'p>(p: AnyDecoder<'p, P>, _ctx: Context) -> anyhow::Result<Self> {
                match p.decode(DecodeHint::Primitive(PrimitiveType::$v))? {
                    DecoderView::Primitive(Primitive::$v(x)) => Ok(x),
                    unexpected => unexpected.mismatch(std::stringify!($t))?,
                }
            }
        }
    };
}

derive_number!(u8, U8);
derive_number!(u16, U16);
derive_number!(u32, U32);
derive_number!(u64, U64);
derive_number!(u128, U128);

derive_number!(i8, I8);
derive_number!(i16, I16);
derive_number!(i32, I32);
derive_number!(i64, I64);
derive_number!(i128, I128);

derive_number!(f32, F32);
derive_number!(f64, F64);

derive_number!(char, Char);

derive_number!(bool, Bool);

impl<P: Decoder> Deserialize<P> for usize {
    fn deserialize<'p>(p: AnyDecoder<'p, P>, _ctx: Context) -> anyhow::Result<Self> {
        match p.decode(DecodeHint::Primitive(PrimitiveType::U64))? {
            DecoderView::Primitive(x) => Ok(x.try_into()?),
            unexpected => unexpected.mismatch(std::stringify!(usize))?,
        }
    }
}

impl<P: Decoder> Deserialize<P> for isize {
    fn deserialize<'p>(p: AnyDecoder<'p, P>, _ctx: Context) -> anyhow::Result<Self> {
        match p.decode(DecodeHint::Primitive(PrimitiveType::I64))? {
            DecoderView::Primitive(x) => Ok(x.try_into()?),
            unexpected => unexpected.mismatch(std::stringify!(isize))?,
        }
    }
}
