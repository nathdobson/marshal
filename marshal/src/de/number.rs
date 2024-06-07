use crate::context::Context;
use crate::de::Deserialize;
use marshal_core::parse::{AnyParser, ParseHint, Parser, ParserView};
use marshal_core::{Primitive, PrimitiveType};

macro_rules! derive_number {
    ($t:ty, $v:ident) => {
        impl<'de, P: Parser<'de>> Deserialize<'de, P> for $t {
            fn deserialize<'p>(p: P::AnyParser<'p>, _ctx: &mut Context) -> anyhow::Result<Self> {
                match p.parse(ParseHint::Primitive(PrimitiveType::$v))? {
                    ParserView::Primitive(Primitive::$v(x)) => Ok(x),
                    unexpected => unexpected.mismatch("u32")?,
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
