use marshal_core::encode::{AnyEncoder, Encoder};
use marshal_core::Primitive;

use crate::context::Context;
use crate::ser::Serialize;

impl<W: Encoder> Serialize<W> for () {
    fn serialize<'w, 'en>(
        &self,
        w: AnyEncoder<'w, 'en, W>,
        _ctx: Context,
    ) -> anyhow::Result<()> {
        w.encode_prim(Primitive::Unit)
    }
}

macro_rules! derive_tuple {
    ($($T:ident),*) => {
        impl<W: Encoder, $( $T: Serialize<W> ),*> Serialize<W>
        for ($($T,)*)
        {
            fn serialize<'w,'en>(&self, w: $crate::encode::AnyEncoder<'w,'en, W>, mut ctx: Context) -> anyhow::Result<()> {
                let mut w = w.encode_tuple(${count($T)})?;
                $(
                    ${ignore($T)}
                    self.${index()}.serialize(w.encode_element()?, ctx.reborrow())?;
                )*
                w.end()?;
                Ok(())
            }
        }
    };
}

macro_rules! derive_tuples {
    ($T1:ident) => {
        derive_tuple!($T1);
    };
    ($T1:ident, $( $TS:ident),* ) =>{
        derive_tuple!($T1, $($TS),*);
        derive_tuples!($($TS),*);
    }
}

derive_tuples!(T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12);
