use marshal_core::{Primitive, PrimitiveType, SchemaError};
use marshal_core::decode::{AnyDecoder, DecodeHint, Decoder, DecoderView};

use crate::context::Context;
use crate::de::Deserialize;
use marshal_core::encode::{AnyEncoder, Encoder};
use crate::ser::Serialize;


impl<D: Decoder> Deserialize<D> for () {
    fn deserialize<'p, 'de>(p: AnyDecoder<'p, 'de, D>, _ctx: Context) -> anyhow::Result<Self> {
        match p.decode(DecodeHint::Primitive(PrimitiveType::Unit))? {
            DecoderView::Primitive(Primitive::Unit) => Ok(()),
            unexpected => unexpected.mismatch("unit")?,
        }
    }
}
macro_rules! derive_tuple {
    ($($T:ident),*) => {
        impl<
            D: Decoder,
            $( $T: Deserialize<D>, )*
        > Deserialize<D> for ($($T,)*)
        {
            fn deserialize<'p, 'de>(d: AnyDecoder<'p, 'de, D>, mut ctx: Context) -> anyhow::Result<Self> {
                match d.decode(DecodeHint::Tuple { len: ${count($T)} })? {
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
                            return Err(SchemaError::TupleTooLong{expected:${count($T)}}.into());
                        }
                        Ok(result)
                    }
                    unexpected => unexpected.mismatch("seq")?,
                }
            }
        }
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



impl<W: Encoder> Serialize<W> for () {
    fn serialize<'w, 'en>(
        &self,
        w: AnyEncoder<'w, 'en, W>,
        _ctx: Context,
    ) -> anyhow::Result<()> {
        w.encode_prim(Primitive::Unit)
    }
}
