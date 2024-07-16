pub mod full;

use crate::{DiscriminantWidth, FixedError};
use marshal::decode::{DecodeHint, DecodeVariantHint, SimpleDecoderView, SpecDecoder};
use marshal::{Primitive, PrimitiveType};
use marshal_vu128::{ReadVu128, VU128_PADDING};
use std::borrow::Cow;

pub struct SimpleFixedSpecDecoder<'de> {
    data: &'de [u8],
}

impl<'de> SimpleFixedSpecDecoder<'de> {
    #[inline]
    pub fn new(data: &'de [u8]) -> Self {
        SimpleFixedSpecDecoder { data }
    }
    #[inline]
    fn decode_prim(&mut self, hint: PrimitiveType) -> anyhow::Result<Primitive> {
        match hint {
            PrimitiveType::Unit => Ok(Primitive::Unit),
            PrimitiveType::Bool => Ok(Primitive::Bool(self.data.read_vu128()?)),
            PrimitiveType::I8 => Ok(Primitive::I8(self.data.read_vu128()?)),
            PrimitiveType::I16 => Ok(Primitive::I16(self.data.read_vu128()?)),
            PrimitiveType::I32 => Ok(Primitive::I32(self.data.read_vu128()?)),
            PrimitiveType::I64 => Ok(Primitive::I64(self.data.read_vu128()?)),
            PrimitiveType::I128 => Ok(Primitive::I128(self.data.read_vu128()?)),
            PrimitiveType::U8 => Ok(Primitive::U8(self.data.read_vu128()?)),
            PrimitiveType::U16 => Ok(Primitive::U16(self.data.read_vu128()?)),
            PrimitiveType::U32 => Ok(Primitive::U32(self.data.read_vu128()?)),
            PrimitiveType::U64 => Ok(Primitive::U64(self.data.read_vu128()?)),
            PrimitiveType::U128 => Ok(Primitive::U128(self.data.read_vu128()?)),
            PrimitiveType::F32 => Ok(Primitive::F32(self.data.read_vu128()?)),
            PrimitiveType::F64 => Ok(Primitive::F64(self.data.read_vu128()?)),
            PrimitiveType::Char => Ok(Primitive::Char(self.data.read_vu128::<u32>()?.try_into()?)),
        }
    }
    #[inline]
    fn decode_str(&mut self) -> anyhow::Result<Cow<'de, str>> {
        let len = usize::try_from(self.data.read_vu128::<u64>()?)?;
        Ok(Cow::Borrowed(std::str::from_utf8(
            self.data.take(..len).ok_or(FixedError::UnexpectedEof)?,
        )?))
    }
    #[inline]
    fn decode_bytes(&mut self) -> anyhow::Result<Cow<'de, [u8]>> {
        let len = usize::try_from(self.data.read_vu128::<u64>()?)?;
        Ok(Cow::Borrowed(
            self.data.take(..len).ok_or(FixedError::UnexpectedEof)?,
        ))
    }
    #[inline]
    fn decode_discriminant(&mut self, variants: usize) -> anyhow::Result<usize> {
        Ok(match DiscriminantWidth::from_max(variants) {
            DiscriminantWidth::U8 => self.data.read_vu128::<u8>()? as usize,
            DiscriminantWidth::U16 => self.data.read_vu128::<u16>()? as usize,
            DiscriminantWidth::U32 => self.data.read_vu128::<u32>()? as usize,
            DiscriminantWidth::U64 => self.data.read_vu128::<u64>()? as usize,
        })
    }
    #[inline]
    pub fn end(self) -> anyhow::Result<()> {
        if self.data.len() > VU128_PADDING {
            return Err(FixedError::TrailingData.into());
        }
        if self.data.iter().any(|x| *x != 0) {
            return Err(FixedError::NonZeroPadding.into());
        }
        Ok(())
    }
}

pub enum FixedAnyDecoder {
    Any,
    Discriminant(usize),
}

pub struct FixedSeqDecoder {
    len: usize,
}

pub struct FixedMapDecoder {
    len: usize,
}

pub struct FixedDiscriminantDecoder {
    discriminant: usize,
}

impl<'de> SpecDecoder<'de> for SimpleFixedSpecDecoder<'de> {
    type AnyDecoder = FixedAnyDecoder;
    type SeqDecoder = FixedSeqDecoder;
    type MapDecoder = FixedMapDecoder;
    type KeyDecoder = ();
    type ValueDecoder = ();
    type DiscriminantDecoder = FixedDiscriminantDecoder;
    type VariantDecoder = ();
    type EnumCloser = ();
    type SomeDecoder = ();
    type SomeCloser = ();

    #[inline]
    fn decode(
        &mut self,
        any: Self::AnyDecoder,
        hint: DecodeHint,
    ) -> anyhow::Result<SimpleDecoderView<'de, Self>> {
        match any {
            FixedAnyDecoder::Any => {}
            FixedAnyDecoder::Discriminant(disc) => match hint {
                DecodeHint::Primitive(p) => {
                    return Ok(SimpleDecoderView::Primitive(match p {
                        PrimitiveType::I8 => Primitive::I8(disc.try_into()?),
                        PrimitiveType::I16 => Primitive::I16(disc.try_into()?),
                        PrimitiveType::I32 => Primitive::I32(disc.try_into()?),
                        PrimitiveType::I64 => Primitive::I64(disc.try_into()?),
                        PrimitiveType::I128 => Primitive::I128(disc.try_into()?),
                        PrimitiveType::U8 => Primitive::U8(disc.try_into()?),
                        PrimitiveType::U16 => Primitive::U16(disc.try_into()?),
                        PrimitiveType::U32 => Primitive::U32(disc.try_into()?),
                        PrimitiveType::U64 => Primitive::U64(disc.try_into()?),
                        PrimitiveType::U128 => Primitive::U128(disc.try_into()?),
                        _ => return Err(FixedError::UnsupportedHint.into()),
                    }))
                }
                DecodeHint::Identifier => {}
                _ => return Err(FixedError::UnsupportedHint.into()),
            },
        }
        match hint {
            DecodeHint::Any => Err(FixedError::UnsupportedHint.into()),
            DecodeHint::Primitive(x) => Ok(SimpleDecoderView::Primitive(self.decode_prim(x)?)),
            DecodeHint::String => Ok(SimpleDecoderView::String(self.decode_str()?)),
            DecodeHint::Bytes => Ok(SimpleDecoderView::Bytes(self.decode_bytes()?)),
            DecodeHint::Option => {
                if self.data.read_vu128::<bool>()? {
                    Ok(SimpleDecoderView::Some(()))
                } else {
                    Ok(SimpleDecoderView::None)
                }
            }
            DecodeHint::UnitStruct { .. } => Ok(SimpleDecoderView::Primitive(Primitive::Unit)),
            DecodeHint::Seq => Ok(SimpleDecoderView::Seq(FixedSeqDecoder {
                len: self.data.read_vu128::<u64>()? as usize,
            })),
            DecodeHint::Tuple { len } => Ok(SimpleDecoderView::Seq(FixedSeqDecoder { len })),
            DecodeHint::TupleStruct { name: _, len } => {
                Ok(SimpleDecoderView::Seq(FixedSeqDecoder { len }))
            }
            DecodeHint::Map => Ok(SimpleDecoderView::Map(FixedMapDecoder {
                len: self.data.read_vu128::<u64>()? as usize,
            })),
            DecodeHint::Struct { name: _, fields } => Ok(SimpleDecoderView::Seq(FixedSeqDecoder {
                len: fields.len(),
            })),
            DecodeHint::Enum { name: _, variants } => {
                Ok(SimpleDecoderView::Enum(FixedDiscriminantDecoder {
                    discriminant: self.decode_discriminant(variants.len())?,
                }))
            }
            DecodeHint::Identifier => Err(FixedError::UnsupportedHint.into()),
            DecodeHint::Ignore => Err(FixedError::UnsupportedHint.into()),
        }
    }

    #[inline]
    fn is_human_readable(&self) -> bool {
        false
    }

    #[inline]
    fn decode_seq_next(
        &mut self,
        seq: &mut Self::SeqDecoder,
    ) -> anyhow::Result<Option<Self::AnyDecoder>> {
        if let Some(new_len) = seq.len.checked_sub(1) {
            seq.len = new_len;
            Ok(Some(FixedAnyDecoder::Any))
        } else {
            Ok(None)
        }
    }

    #[inline]
    fn decode_seq_end(&mut self, _: Self::SeqDecoder) -> anyhow::Result<()> {
        Ok(())
    }

    #[inline]
    fn decode_map_next(
        &mut self,
        map: &mut Self::MapDecoder,
    ) -> anyhow::Result<Option<Self::KeyDecoder>> {
        if let Some(new_len) = map.len.checked_sub(1) {
            map.len = new_len;
            Ok(Some(()))
        } else {
            Ok(None)
        }
    }

    #[inline]
    fn decode_map_end(&mut self, _: Self::MapDecoder) -> anyhow::Result<()> {
        Ok(())
    }

    #[inline]
    fn decode_entry_key(
        &mut self,
        _: Self::KeyDecoder,
    ) -> anyhow::Result<(Self::AnyDecoder, Self::ValueDecoder)> {
        Ok((FixedAnyDecoder::Any, ()))
    }

    #[inline]
    fn decode_entry_value(&mut self, _: Self::ValueDecoder) -> anyhow::Result<Self::AnyDecoder> {
        Ok(FixedAnyDecoder::Any)
    }

    #[inline]
    fn decode_enum_discriminant(
        &mut self,
        e: Self::DiscriminantDecoder,
    ) -> anyhow::Result<(Self::AnyDecoder, Self::VariantDecoder)> {
        Ok((FixedAnyDecoder::Discriminant(e.discriminant), ()))
    }

    #[inline]
    fn decode_enum_variant(
        &mut self,
        _: Self::VariantDecoder,
        hint: DecodeVariantHint,
    ) -> anyhow::Result<(SimpleDecoderView<'de, Self>, Self::EnumCloser)> {
        match hint {
            DecodeVariantHint::UnitVariant => {
                Ok((SimpleDecoderView::Primitive(Primitive::Unit), ()))
            }
            DecodeVariantHint::TupleVariant { len } => {
                Ok((SimpleDecoderView::Seq(FixedSeqDecoder { len }), ()))
            }
            DecodeVariantHint::StructVariant { fields } => Ok((
                SimpleDecoderView::Seq(FixedSeqDecoder { len: fields.len() }),
                (),
            )),
            DecodeVariantHint::Ignore => Err(FixedError::UnsupportedHint.into()),
        }
    }

    #[inline]
    fn decode_enum_end(&mut self, _: Self::EnumCloser) -> anyhow::Result<()> {
        Ok(())
    }

    #[inline]
    fn decode_some_inner(
        &mut self,
        _: Self::SomeDecoder,
    ) -> anyhow::Result<(Self::AnyDecoder, Self::SomeCloser)> {
        Ok((FixedAnyDecoder::Any, ()))
    }

    #[inline]
    fn decode_some_end(&mut self, _: Self::SomeCloser) -> anyhow::Result<()> {
        Ok(())
    }
}
