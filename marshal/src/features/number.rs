use marshal_core::{Primitive, PrimitiveType};
use marshal_core::decode::{AnyDecoder, DecodeHint, Decoder, DecoderView};
use marshal_core::encode::{AnyEncoder, Encoder};
use crate::context::Context;
use crate::de::Deserialize;
use crate::ser::Serialize;

macro_rules! derive_number {
    ($t:ty, $v:ident) => {
        impl<D: Decoder> Deserialize<D> for $t {
            fn deserialize<'p, 'de>(d: AnyDecoder<'p, 'de, D>, _ctx: Context) -> anyhow::Result<Self> {
                match d.decode(DecodeHint::Primitive(PrimitiveType::$v))? {
                    DecoderView::Primitive(Primitive::$v(x)) => Ok(x),
                    unexpected => unexpected.mismatch(std::stringify!($t))?,
                }
            }
        }
        impl<W: Encoder> Serialize<W> for $t {
            fn serialize<'w, 'en>(
                &self,
                w: $crate::encode::AnyEncoder<'w, 'en, W>,
                _ctx: Context,
            ) -> anyhow::Result<()> {
                w.encode_prim(Primitive::$v(*self))
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

impl<D: Decoder> Deserialize<D> for usize {
    fn deserialize<'p,'de>(p: AnyDecoder<'p, 'de, D>, _ctx: Context) -> anyhow::Result<Self> {
        match p.decode(DecodeHint::Primitive(PrimitiveType::U64))? {
            DecoderView::Primitive(x) => Ok(x.try_into()?),
            unexpected => unexpected.mismatch(std::stringify!(usize))?,
        }
    }
}

impl<D:Decoder> Deserialize<D> for isize {
    fn deserialize<'p,'de>(p: AnyDecoder<'p, 'de, D>, _ctx: Context) -> anyhow::Result<Self> {
        match p.decode(DecodeHint::Primitive(PrimitiveType::I64))? {
            DecoderView::Primitive(x) => Ok(x.try_into()?),
            unexpected => unexpected.mismatch(std::stringify!(isize))?,
        }
    }
}

impl<W: Encoder> Serialize<W> for usize {
    fn serialize<'w, 'en>(
        &self,
        w: AnyEncoder<'w, 'en, W>,
        _ctx: Context,
    ) -> anyhow::Result<()> {
        w.encode_prim(Primitive::U64(*self as u64))
    }
}

impl<W: Encoder> Serialize<W> for isize {
    fn serialize<'w, 'en>(
        &self,
        w: AnyEncoder<'w, 'en, W>,
        _ctx: Context,
    ) -> anyhow::Result<()> {
        w.encode_prim(Primitive::I64(*self as i64))
    }
}