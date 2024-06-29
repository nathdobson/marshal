use marshal_core::encode::{AnyEncoder, Encoder};
use marshal_core::Primitive;

use crate::context::Context;
use crate::ser::Serialize;

macro_rules! derive_number {
    ($t:ty, $v:ident) => {
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
