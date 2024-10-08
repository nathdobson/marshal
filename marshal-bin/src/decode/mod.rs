use std::borrow::Cow;
use std::fmt::{Debug, Display, Formatter};

use by_address::ByAddress;
use num_traits::FromPrimitive;
use safe_once_map::cell::OnceCellMap;

use crate::util::StableCellVec;
use crate::{TypeTag};
use marshal_core::decode::{DecodeHint, DecodeVariantHint, SimpleDecoderView, SpecDecoder};
use marshal_core::{Primitive, PrimitiveType};
use marshal_vu128::{ReadVu128, VU128_PADDING};

pub mod full;

type EnumDefNative = &'static [&'static str];

enum EnumDefKey {
    Native(usize),
    Foreign(String),
}

struct EnumDefTranslation {
    keys: Vec<EnumDefKey>,
}

struct EnumDefForeign {
    fields: Vec<String>,
    default_translation: EnumDefTranslation,
    custom_translation: OnceCellMap<ByAddress<EnumDefNative>, EnumDefTranslation>,
}

pub struct BinDecoderSchema {
    enum_defs: StableCellVec<EnumDefForeign>,
}

impl BinDecoderSchema {
    pub fn new() -> Self {
        BinDecoderSchema {
            enum_defs: StableCellVec::new(),
        }
    }
}

pub struct SimpleBinSpecDecoder<'de> {
    content: &'de [u8],
    schema: &'de BinDecoderSchema,
}

#[derive(Debug)]
pub enum BinDecoderError {
    TrailingData,
    NonZeroPadding,
    Eof,
    BadTag(u8),
    NoSuchEnumDef,
    MissingField(&'static str),
}

impl Display for BinDecoderError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            BinDecoderError::TrailingData => write!(f, "input contains excessive trailing bytes"),
            BinDecoderError::NonZeroPadding => write!(f, "final padding is not zero"),
            BinDecoderError::Eof => write!(f, "unexpected end of file"),
            BinDecoderError::BadTag(x) => write!(f, "unknown type tag {}", x),
            BinDecoderError::NoSuchEnumDef => write!(f, "reference to unknown enum definition (did you remember to reuse the BinDecoderSchema?)"),
            BinDecoderError::MissingField(field) => write!(f, "attempted to deserialize struct with missing field `{}'", field),
        }
    }
}

impl std::error::Error for BinDecoderError {}

impl<'de> SimpleBinSpecDecoder<'de> {
    #[inline]
    pub fn new(data: &'de [u8], schema: &'de mut BinDecoderSchema) -> SimpleBinSpecDecoder<'de> {
        SimpleBinSpecDecoder {
            content: data,
            schema,
        }
    }
    #[inline]
    pub fn end(self) -> anyhow::Result<()> {
        if self.content.len() > VU128_PADDING {
            return Err(BinDecoderError::TrailingData.into());
        }
        if self.content.iter().any(|x| *x != 0) {
            return Err(BinDecoderError::NonZeroPadding.into());
        }
        Ok(())
    }
}

impl<'de> SimpleBinSpecDecoder<'de> {
    #[inline]
    fn read_count(&mut self, count: usize) -> anyhow::Result<&'de [u8]> {
        Ok(self.content.take(..count).ok_or(BinDecoderError::Eof)?)
    }
    #[inline]
    fn read_usize(&mut self) -> anyhow::Result<usize> {
        Ok(usize::try_from(self.content.read_vu128::<u64>()?)?)
    }
    #[inline]
    fn decode_type_tag(&mut self) -> anyhow::Result<TypeTag> {
        let tag_num = self.read_count(1)?[0];
        Ok(TypeTag::from_u8(tag_num).ok_or(BinDecoderError::BadTag(tag_num))?)
    }
    #[inline]
    fn read_bytes(&mut self) -> anyhow::Result<&'de [u8]> {
        let len = self.read_usize()?;
        self.read_count(len)
    }
    #[inline]
    fn read_str(&mut self) -> anyhow::Result<&'de str> {
        Ok(std::str::from_utf8(self.read_bytes()?)?)
    }
    fn read_enum_def(&mut self) -> anyhow::Result<()> {
        let count = self.read_usize()?;
        let fields = (0..count)
            .map(|_| Ok(self.read_str()?.to_string()))
            .collect::<anyhow::Result<Vec<_>>>()?;
        let def = EnumDefForeign {
            fields,
            default_translation: EnumDefTranslation { keys: vec![] },
            custom_translation: Default::default(),
        };
        self.schema.enum_defs.push(def);
        Ok(())
    }
    #[inline]
    fn read_enum_def_ref(&mut self) -> anyhow::Result<&'de EnumDefForeign> {
        let index = self.read_usize()?;
        Ok(self
            .schema
            .enum_defs
            .get(index)
            .ok_or(BinDecoderError::NoSuchEnumDef)?)
    }
}

impl EnumDefForeign {
    fn get_translation<'s>(&'s self, native: Option<EnumDefNative>) -> &'s EnumDefTranslation {
        match native {
            None => &self.default_translation,
            Some(native) => self
                .custom_translation
                .get_or_insert(Cow::Owned(ByAddress(native)))
                .get_or_init(|| EnumDefTranslation {
                    keys: self
                        .fields
                        .iter()
                        .map(|foreign| {
                            if let Some(native) = native.iter().position(|native| native == foreign)
                            {
                                EnumDefKey::Native(native)
                            } else {
                                EnumDefKey::Foreign(foreign.clone())
                            }
                        })
                        .collect(),
                }),
        }
    }
}

impl<'s> Default for BinAnyDecoder<'s> {
    fn default() -> Self {
        BinAnyDecoder::Read
    }
}

pub enum BinAnyDecoder<'de> {
    U32(u32),
    Str(&'de str),
    Read,
}

pub struct BinSeqDecoder {
    len: usize,
}

pub struct BinMapDecoder<'de>(BinMapDecoderInner<'de>);

enum BinMapDecoderInner<'de> {
    WithSchema(&'de [EnumDefKey]),
    WithLength(usize),
}

pub enum BinKeyDecoder<'de> {
    Foreign(&'de str),
    Native(usize),
    Read,
}

pub struct BinDiscriminantDecoder<'de> {
    variant: &'de EnumDefKey,
}

impl<'de> SpecDecoder<'de> for SimpleBinSpecDecoder<'de> {
    type AnyDecoder = BinAnyDecoder<'de>;
    type SeqDecoder = BinSeqDecoder;
    type MapDecoder = BinMapDecoder<'de>;
    type KeyDecoder = BinKeyDecoder<'de>;
    type ValueDecoder = ();
    type DiscriminantDecoder = BinDiscriminantDecoder<'de>;
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
            BinAnyDecoder::U32(x) => return Ok(SimpleDecoderView::Primitive(Primitive::U32(x))),
            BinAnyDecoder::Str(x) => {
                return Ok(SimpleDecoderView::String(Cow::Owned(x.to_string())));
            }
            BinAnyDecoder::Read => {}
        }
        loop {
            let tag = self.decode_type_tag()?;
            let () = match tag {
                TypeTag::Unit => return Ok(SimpleDecoderView::Primitive(Primitive::Unit)),
                TypeTag::Bool => {
                    return Ok(SimpleDecoderView::Primitive(Primitive::Bool(
                        self.content.read_vu128()?,
                    )));
                }
                TypeTag::I8 => {
                    return Ok(SimpleDecoderView::Primitive(Primitive::I8(
                        self.content.read_vu128()?,
                    )));
                }
                TypeTag::I16 => {
                    return Ok(SimpleDecoderView::Primitive(Primitive::I16(
                        self.content.read_vu128()?,
                    )));
                }
                TypeTag::I32 => {
                    return Ok(SimpleDecoderView::Primitive(Primitive::I32(
                        self.content.read_vu128()?,
                    )));
                }
                TypeTag::I64 => {
                    return Ok(SimpleDecoderView::Primitive(Primitive::I64(
                        self.content.read_vu128()?,
                    )));
                }
                TypeTag::I128 => {
                    return Ok(SimpleDecoderView::Primitive(Primitive::I128(
                        self.content.read_vu128()?,
                    )));
                }
                TypeTag::U8 => {
                    return Ok(SimpleDecoderView::Primitive(Primitive::U8(
                        self.content.read_vu128()?,
                    )));
                }
                TypeTag::U16 => {
                    return Ok(SimpleDecoderView::Primitive(Primitive::U16(
                        self.content.read_vu128()?,
                    )));
                }
                TypeTag::U32 => {
                    return Ok(SimpleDecoderView::Primitive(Primitive::U32(
                        self.content.read_vu128()?,
                    )));
                }
                TypeTag::U64 => {
                    return Ok(SimpleDecoderView::Primitive(Primitive::U64(
                        self.content.read_vu128()?,
                    )));
                }
                TypeTag::U128 => {
                    return Ok(SimpleDecoderView::Primitive(Primitive::U128(
                        self.content.read_vu128()?,
                    )));
                }
                TypeTag::F32 => {
                    return Ok(SimpleDecoderView::Primitive(Primitive::F32(
                        self.content.read_vu128()?,
                    )));
                }
                TypeTag::F64 => {
                    return Ok(SimpleDecoderView::Primitive(Primitive::F32(
                        self.content.read_vu128()?,
                    )));
                }
                TypeTag::Char => {
                    return Ok(SimpleDecoderView::Primitive(Primitive::Char(
                        self.content.read_vu128::<u32>()?.try_into()?,
                    )));
                }
                TypeTag::Struct => {
                    let enum_def = self.read_enum_def_ref()?;
                    let fields = match hint {
                        DecodeHint::Struct { name: _, fields } => Some(fields),
                        _ => None,
                    };
                    let trans = enum_def.get_translation(fields);
                    return Ok(SimpleDecoderView::Map(BinMapDecoder(
                        BinMapDecoderInner::WithSchema(&trans.keys),
                    )));
                }
                TypeTag::TupleStruct => {
                    let len = self.read_usize()?;
                    return Ok(SimpleDecoderView::Seq(BinSeqDecoder { len }));
                }
                TypeTag::Enum => {
                    let enum_def = self.read_enum_def_ref()?;
                    let variant = self.read_usize()?;
                    let variants = match hint {
                        DecodeHint::Enum { name: _, variants } => Some(variants),
                        _ => None,
                    };
                    let variant = &enum_def.get_translation(variants).keys[variant];
                    return Ok(SimpleDecoderView::Enum(BinDiscriminantDecoder { variant }));
                }
                TypeTag::Seq => {
                    let len = self.read_usize()?;
                    return Ok(SimpleDecoderView::Seq(BinSeqDecoder { len }));
                }
                TypeTag::Map => {
                    let len = self.read_usize()?;
                    return Ok(SimpleDecoderView::Map(BinMapDecoder(
                        BinMapDecoderInner::WithLength(len),
                    )));
                }
                TypeTag::Tuple => {
                    let len = self.content.read_vu128::<u64>()?;
                    return Ok(SimpleDecoderView::Seq(BinSeqDecoder {
                        len: usize::try_from(len)?,
                    }));
                }
                TypeTag::EnumDef => self.read_enum_def()?,
                TypeTag::String => return Ok(SimpleDecoderView::String(self.read_str()?.into())),
                TypeTag::UnitStruct => return Ok(SimpleDecoderView::Primitive(Primitive::Unit)),
                TypeTag::Bytes => return Ok(SimpleDecoderView::Bytes(self.read_bytes()?.into())),
                TypeTag::None => return Ok(SimpleDecoderView::None),
                TypeTag::Some => return Ok(SimpleDecoderView::Some(())),
            };
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
        if let Some(len2) = seq.len.checked_sub(1) {
            seq.len = len2;
            Ok(Some(BinAnyDecoder::Read))
        } else {
            Ok(None)
        }
    }

    #[inline]
    fn decode_seq_end(&mut self, _seq: Self::SeqDecoder) -> anyhow::Result<()> {
        Ok(())
    }

    #[inline]
    fn decode_map_next(
        &mut self,
        map: &mut Self::MapDecoder,
    ) -> anyhow::Result<Option<Self::KeyDecoder>> {
        match &mut map.0 {
            BinMapDecoderInner::WithSchema(schema) => {
                if let Some(key) = schema.take_first() {
                    match key {
                        EnumDefKey::Native(x) => Ok(Some(BinKeyDecoder::Native(*x))),
                        EnumDefKey::Foreign(x) => Ok(Some(BinKeyDecoder::Foreign(x))),
                    }
                } else {
                    Ok(None)
                }
            }
            BinMapDecoderInner::WithLength(len) => {
                if let Some(l2) = len.checked_sub(1) {
                    *len = l2;
                    Ok(Some(BinKeyDecoder::Read))
                } else {
                    Ok(None)
                }
            }
        }
    }

    #[inline]
    fn decode_map_end(&mut self, _map: Self::MapDecoder) -> anyhow::Result<()> {
        Ok(())
    }

    #[inline]
    fn decode_entry_key(
        &mut self,
        key: Self::KeyDecoder,
    ) -> anyhow::Result<(Self::AnyDecoder, Self::ValueDecoder)> {
        match key {
            BinKeyDecoder::Foreign(x) => Ok((BinAnyDecoder::Str(x), ())),
            BinKeyDecoder::Native(x) => Ok((BinAnyDecoder::U32(u32::try_from(x)?), ())),
            BinKeyDecoder::Read => Ok((BinAnyDecoder::Read, ())),
        }
    }

    #[inline]
    fn decode_entry_value(&mut self, _: Self::ValueDecoder) -> anyhow::Result<Self::AnyDecoder> {
        Ok(BinAnyDecoder::Read)
    }

    #[inline]
    fn decode_enum_discriminant(
        &mut self,
        e: Self::DiscriminantDecoder,
    ) -> anyhow::Result<(Self::AnyDecoder, Self::VariantDecoder)> {
        Ok((
            match e.variant {
                EnumDefKey::Native(x) => BinAnyDecoder::U32(u32::try_from(*x)?),
                EnumDefKey::Foreign(y) => BinAnyDecoder::Str(y),
            },
            (),
        ))
    }

    #[inline]
    fn decode_enum_variant(
        &mut self,
        _: Self::VariantDecoder,
        hint: DecodeVariantHint,
    ) -> anyhow::Result<(SimpleDecoderView<'de, Self>, Self::EnumCloser)> {
        Ok((
            self.decode(
                BinAnyDecoder::Read,
                match hint {
                    DecodeVariantHint::UnitVariant => DecodeHint::Primitive(PrimitiveType::Unit),
                    DecodeVariantHint::TupleVariant { len } => DecodeHint::TupleStruct {
                        name: "<enum>",
                        len,
                    },
                    DecodeVariantHint::StructVariant { fields } => DecodeHint::Struct {
                        name: "<enum>",
                        fields,
                    },
                    DecodeVariantHint::Ignore => DecodeHint::Ignore,
                },
            )?,
            (),
        ))
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
        Ok((BinAnyDecoder::Read, ()))
    }

    #[inline]
    fn decode_some_end(&mut self, _: Self::SomeCloser) -> anyhow::Result<()> {
        Ok(())
    }
}
