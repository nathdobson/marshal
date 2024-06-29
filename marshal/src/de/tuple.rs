use marshal_core::decode::{AnyDecoder, DecodeHint, Decoder, DecoderView};
use marshal_core::{Primitive, PrimitiveType};

use crate::context::Context;
use crate::de::{Deserialize, SchemaError};

impl<P: Decoder> Deserialize<P> for () {
    fn deserialize(p: AnyDecoder<'_, P>, _ctx: Context) -> anyhow::Result<Self> {
        match p.decode(DecodeHint::Primitive(PrimitiveType::Unit))? {
            DecoderView::Primitive(Primitive::Unit) => Ok(()),
            unexpected => unexpected.mismatch("unit")?,
        }
    }
}
macro_rules! derive_tuple {
    ($($T:ident),*) => {
        impl<
            P: Decoder,
            $( $T: Deserialize< P>, )*
        > Deserialize< P> for ($($T,)*)
        {
            fn deserialize(p: AnyDecoder<'_, P>, mut ctx: Context) -> anyhow::Result<Self> {
                match p.decode(DecodeHint::Tuple { len: 4 })? {
                    DecoderView::Seq(mut p) => {
                        let result=(
                            $(
                                {
                                    let x = $T::deserialize(p.decode_next()?.ok_or(SchemaError::TupleTooShort)?, ctx.reborrow())?;
                                    x
                                },
                            )*
                        );
                        if let Some(_) = p.decode_next()? {
                            return Err(SchemaError::TupleTooLong.into());
                        }
                        Ok(result)
                    }
                    unexpected => unexpected.mismatch("seq")?,
                }
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
