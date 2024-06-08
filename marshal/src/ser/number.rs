use crate::context::Context;
use crate::ser::Serialize;
use marshal_core::encode::{AnyWriter, Writer};
use marshal_core::Primitive;

macro_rules! derive_number {
    ($t:ty, $v:ident) => {
        impl<W: Writer> Serialize<W> for $t {
            fn serialize(&self, w: W::AnyWriter<'_>, _ctx: &mut Context) -> anyhow::Result<()> {
                w.write_prim(Primitive::$v(*self))
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
