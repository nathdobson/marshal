use marshal_core::decode::{AnyDecoder, DecodeHint, Decoder, DecoderView, SeqDecoder};
use marshal_core::{Primitive, PrimitiveType};

use crate::context::Context;
use crate::de::{Deserialize, SchemaError};

impl<'de, P: Decoder<'de>> Deserialize<'de, P> for () {
    fn deserialize(p: P::AnyDecoder<'_>, _: &mut Context) -> anyhow::Result<Self> {
        match p.decode(DecodeHint::Primitive(PrimitiveType::Unit))? {
            DecoderView::Primitive(Primitive::Unit) => Ok(()),
            unexpected => unexpected.mismatch("unit")?,
        }
    }
}

impl<'de, P: Decoder<'de>, T1: Deserialize<'de, P>> Deserialize<'de, P> for (T1, ) {
    fn deserialize(p: P::AnyDecoder<'_>, context: &mut Context) -> anyhow::Result<Self> {
        match p.decode(DecodeHint::Tuple { len: 1 })? {
            DecoderView::Seq(mut p) => {
                let f0 =
                    T1::deserialize(p.decode_next()?.ok_or(SchemaError::TupleTooShort)?, context)?;
                if let Some(_) = p.decode_next()? {
                    return Err(SchemaError::TupleTooLong.into());
                }
                Ok((f0, ))
            }
            unexpected => unexpected.mismatch("seq")?,
        }
    }
}

impl<'de, P: Decoder<'de>, T1: Deserialize<'de, P>, T2: Deserialize<'de, P>> Deserialize<'de, P>
for (T1, T2)
{
    fn deserialize(p: P::AnyDecoder<'_>, context: &mut Context) -> anyhow::Result<Self> {
        match p.decode(DecodeHint::Tuple { len: 2 })? {
            DecoderView::Seq(mut p) => {
                let f0 =
                    T1::deserialize(p.decode_next()?.ok_or(SchemaError::TupleTooShort)?, context)?;
                let f1 =
                    T2::deserialize(p.decode_next()?.ok_or(SchemaError::TupleTooShort)?, context)?;
                if let Some(_) = p.decode_next()? {
                    return Err(SchemaError::TupleTooLong.into());
                }
                Ok((f0, f1))
            }
            unexpected => unexpected.mismatch("seq")?,
        }
    }
}

impl<
    'de,
    P: Decoder<'de>,
    T1: Deserialize<'de, P>,
    T2: Deserialize<'de, P>,
    T3: Deserialize<'de, P>,
> Deserialize<'de, P> for (T1, T2, T3)
{
    fn deserialize(p: P::AnyDecoder<'_>, context: &mut Context) -> anyhow::Result<Self> {
        match p.decode(DecodeHint::Tuple { len: 3 })? {
            DecoderView::Seq(mut p) => {
                let f0 =
                    T1::deserialize(p.decode_next()?.ok_or(SchemaError::TupleTooShort)?, context)?;
                let f1 =
                    T2::deserialize(p.decode_next()?.ok_or(SchemaError::TupleTooShort)?, context)?;
                let f2 =
                    T3::deserialize(p.decode_next()?.ok_or(SchemaError::TupleTooShort)?, context)?;
                if let Some(_) = p.decode_next()? {
                    return Err(SchemaError::TupleTooLong.into());
                }
                Ok((f0, f1, f2))
            }
            unexpected => unexpected.mismatch("seq")?,
        }
    }
}

impl<
    'de,
    P: Decoder<'de>,
    T1: Deserialize<'de, P>,
    T2: Deserialize<'de, P>,
    T3: Deserialize<'de, P>,
    T4: Deserialize<'de, P>,
> Deserialize<'de, P> for (T1, T2, T3, T4)
{
    fn deserialize(p: P::AnyDecoder<'_>, context: &mut Context) -> anyhow::Result<Self> {
        match p.decode(DecodeHint::Tuple { len: 4 })? {
            DecoderView::Seq(mut p) => {
                let f0 =
                    T1::deserialize(p.decode_next()?.ok_or(SchemaError::TupleTooShort)?, context)?;
                let f1 =
                    T2::deserialize(p.decode_next()?.ok_or(SchemaError::TupleTooShort)?, context)?;
                let f2 =
                    T3::deserialize(p.decode_next()?.ok_or(SchemaError::TupleTooShort)?, context)?;
                let f3 =
                    T4::deserialize(p.decode_next()?.ok_or(SchemaError::TupleTooShort)?, context)?;
                if let Some(_) = p.decode_next()? {
                    return Err(SchemaError::TupleTooLong.into());
                }
                Ok((f0, f1, f2, f3))
            }
            unexpected => unexpected.mismatch("seq")?,
        }
    }
}
