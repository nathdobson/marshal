use std::borrow::Cow;
use std::collections::HashMap;
use std::fmt::{Debug, Display, Formatter};
use std::sync::Arc;

use by_address::ByAddress;
use id_vec::{Id, IdVec};
use lock_api::ArcMutexGuard;
use num_traits::FromPrimitive;
use parking_lot::{Mutex, RawMutex};

use marshal_core::decode::{DecodeHint, DecodeVariantHint, Decoder, SimpleDecoderView};
use marshal_core::{Primitive, PrimitiveType};

use crate::to_from_vu128::{Array, ToFromVu128};
use crate::{TypeTag, VU128_MAX_PADDING};

pub mod full;

type EnumDefNative = &'static [&'static str];

#[derive(Copy, Clone)]
enum EnumDefKey {
    Native(usize),
    Foreign(Id<String>),
}

struct EnumDefTranslation {
    keys: Vec<EnumDefKey>,
}

struct EnumDefForeign {
    fields: Vec<String>,
    default_translation: Id<EnumDefTranslation>,
    custom_translation: HashMap<ByAddress<EnumDefNative>, Id<EnumDefTranslation>>,
}

struct BinDecoderSchemaInner {
    strings: IdVec<String>,
    translations: IdVec<EnumDefTranslation>,
    foreign: IdVec<EnumDefForeign>,
}

pub struct BinDecoderSchema {
    inner: Arc<Mutex<BinDecoderSchemaInner>>,
}

impl BinDecoderSchema {
    pub fn new() -> Self {
        BinDecoderSchema {
            inner: Arc::new(Mutex::new(BinDecoderSchemaInner {
                strings: IdVec::new(),
                translations: IdVec::new(),
                foreign: IdVec::new(),
            })),
        }
    }
}

pub struct SimpleBinDecoder<'de> {
    content: &'de [u8],
    schema: ArcMutexGuard<RawMutex, BinDecoderSchemaInner>,
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

impl<'de> SimpleBinDecoder<'de> {
    pub fn new(data: &'de [u8], schema: &BinDecoderSchema) -> SimpleBinDecoder<'de> {
        SimpleBinDecoder {
            content: data,
            schema: schema.inner.try_lock_arc().unwrap(),
        }
    }
    pub fn end(self) -> anyhow::Result<()> {
        if self.content.len() > VU128_MAX_PADDING {
            return Err(BinDecoderError::TrailingData.into());
        }
        if self.content.iter().any(|x| *x != 0) {
            return Err(BinDecoderError::NonZeroPadding.into());
        }
        Ok(())
    }
}

impl<'de> SimpleBinDecoder<'de> {
    fn read_count(&mut self, count: usize) -> anyhow::Result<&'de [u8]> {
        Ok(self.content.take(..count).ok_or(BinDecoderError::Eof)?)
    }
    fn read_vu128<T: ToFromVu128 + Display>(&mut self) -> anyhow::Result<T> {
        let (value, count) = T::decode_vu128(T::Buffer::try_from_slice(
            &self.content[..T::Buffer::ARRAY_LEN],
        )?);
        self.content.take(..count).ok_or(BinDecoderError::Eof)?;
        Ok(value)
    }
    fn read_usize(&mut self) -> anyhow::Result<usize> {
        Ok(usize::try_from(self.read_vu128::<u64>()?)?)
    }
    fn decode_type_tag(&mut self) -> anyhow::Result<TypeTag> {
        let tag_num = self.read_count(1)?[0];
        Ok(TypeTag::from_u8(tag_num).ok_or(BinDecoderError::BadTag(tag_num))?)
    }
    fn read_bytes(&mut self) -> anyhow::Result<&'de [u8]> {
        let len = self.read_usize()?;
        self.read_count(len)
    }
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
            default_translation: self
                .schema
                .translations
                .insert(EnumDefTranslation { keys: vec![] }),
            custom_translation: Default::default(),
        };
        self.schema.foreign.insert(def);
        Ok(())
    }
    fn read_enum_def_ref(&mut self) -> anyhow::Result<Id<EnumDefForeign>> {
        let index = self.read_usize()?;
        assert!(index < self.schema.foreign.len());
        Ok(Id::from_index(index))
    }
    fn get_translation(
        &mut self,
        foreign: Id<EnumDefForeign>,
        native: Option<EnumDefNative>,
    ) -> Id<EnumDefTranslation> {
        let schema = &mut *self.schema;

        let foreign = &mut schema.foreign[foreign];
        match native {
            None => foreign.default_translation,
            Some(native) => *foreign
                .custom_translation
                .entry(ByAddress(native))
                .or_insert_with(|| {
                    schema.translations.insert(EnumDefTranslation {
                        keys: foreign
                            .fields
                            .iter()
                            .map(|foreign| {
                                if let Some(native) =
                                    native.iter().position(|native| native == foreign)
                                {
                                    EnumDefKey::Native(native)
                                } else {
                                    EnumDefKey::Foreign(schema.strings.insert(foreign.clone()))
                                }
                            })
                            .collect(),
                    })
                }),
        }
    }
}

impl Default for BinAnyDecoder {
    fn default() -> Self {
        BinAnyDecoder::Read
    }
}

pub enum BinAnyDecoder {
    U32(u32),
    Str(Id<String>),
    Read,
}

pub struct BinSeqDecoder {
    len: usize,
}

pub struct BinMapDecoder(BinMapDecoderInner);

enum BinMapDecoderInner {
    WithSchema {
        translation: Id<EnumDefTranslation>,
        index: usize,
    },
    WithLength(usize),
}

pub enum BinKeyDecoder {
    Foreign(Id<String>),
    Native(usize),
    Read,
}

pub struct BinDiscriminantDecoder {
    variant: EnumDefKey,
}

impl<'de> Decoder<'de> for SimpleBinDecoder<'de> {
    type AnyDecoder = BinAnyDecoder;
    type SeqDecoder = BinSeqDecoder;
    type MapDecoder = BinMapDecoder;
    type KeyDecoder = BinKeyDecoder;
    type ValueDecoder = ();
    type DiscriminantDecoder = BinDiscriminantDecoder;
    type VariantDecoder = ();
    type EnumCloser = ();
    type SomeDecoder = ();
    type SomeCloser = ();

    fn decode(
        &mut self,
        any: Self::AnyDecoder,
        hint: DecodeHint,
    ) -> anyhow::Result<SimpleDecoderView<'de, Self>> {
        match any {
            BinAnyDecoder::U32(x) => return Ok(SimpleDecoderView::Primitive(Primitive::U32(x))),
            BinAnyDecoder::Str(x) => {
                return Ok(SimpleDecoderView::String(Cow::Owned(
                    self.schema.strings[x].clone(),
                )));
            }
            BinAnyDecoder::Read => {}
        }
        loop {
            let tag = self.decode_type_tag()?;
            let () = match tag {
                TypeTag::Unit => return Ok(SimpleDecoderView::Primitive(Primitive::Unit)),
                TypeTag::Bool => {
                    return Ok(SimpleDecoderView::Primitive(Primitive::Bool(
                        self.read_vu128()?,
                    )));
                }
                TypeTag::I8 => {
                    return Ok(SimpleDecoderView::Primitive(Primitive::I8(
                        self.read_vu128()?,
                    )));
                }
                TypeTag::I16 => {
                    return Ok(SimpleDecoderView::Primitive(Primitive::I16(
                        self.read_vu128()?,
                    )));
                }
                TypeTag::I32 => {
                    return Ok(SimpleDecoderView::Primitive(Primitive::I32(
                        self.read_vu128()?,
                    )));
                }
                TypeTag::I64 => {
                    return Ok(SimpleDecoderView::Primitive(Primitive::I64(
                        self.read_vu128()?,
                    )));
                }
                TypeTag::I128 => {
                    return Ok(SimpleDecoderView::Primitive(Primitive::I128(
                        self.read_vu128()?,
                    )));
                }
                TypeTag::U8 => {
                    return Ok(SimpleDecoderView::Primitive(Primitive::U8(
                        self.read_vu128()?,
                    )));
                }
                TypeTag::U16 => {
                    return Ok(SimpleDecoderView::Primitive(Primitive::U16(
                        self.read_vu128()?,
                    )));
                }
                TypeTag::U32 => {
                    return Ok(SimpleDecoderView::Primitive(Primitive::U32(
                        self.read_vu128()?,
                    )));
                }
                TypeTag::U64 => {
                    return Ok(SimpleDecoderView::Primitive(Primitive::U64(
                        self.read_vu128()?,
                    )));
                }
                TypeTag::U128 => {
                    return Ok(SimpleDecoderView::Primitive(Primitive::U128(
                        self.read_vu128()?,
                    )));
                }
                TypeTag::F32 => {
                    return Ok(SimpleDecoderView::Primitive(Primitive::F32(
                        self.read_vu128()?,
                    )));
                }
                TypeTag::F64 => {
                    return Ok(SimpleDecoderView::Primitive(Primitive::F32(
                        self.read_vu128()?,
                    )));
                }
                TypeTag::Char => {
                    return Ok(SimpleDecoderView::Primitive(Primitive::Char(
                        self.read_vu128::<u32>()?.try_into()?,
                    )));
                }
                TypeTag::Struct => {
                    let enum_def = self.read_enum_def_ref()?;
                    let fields = match hint {
                        DecodeHint::Struct { name: _, fields } => Some(fields),
                        _ => None,
                    };
                    let trans = self.get_translation(enum_def, fields);
                    return Ok(SimpleDecoderView::Map(BinMapDecoder(
                        BinMapDecoderInner::WithSchema {
                            translation: trans,
                            index: 0,
                        },
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
                    let trans = self.get_translation(enum_def, variants);
                    let variant = self.schema.translations[trans].keys[variant];
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
                    let len = self.read_vu128::<u64>()?;
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

    fn is_human_readable(&self) -> bool {
        false
    }

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

    fn decode_seq_end(&mut self, _seq: Self::SeqDecoder) -> anyhow::Result<()> {
        Ok(())
    }

    fn decode_map_next(
        &mut self,
        map: &mut Self::MapDecoder,
    ) -> anyhow::Result<Option<Self::KeyDecoder>> {
        match &mut map.0 {
            BinMapDecoderInner::WithSchema { translation, index } => {
                let keys = &self.schema.translations[*translation].keys;
                if *index == keys.len() {
                    Ok(None)
                } else {
                    let i = *index;
                    *index += 1;
                    match keys[i] {
                        EnumDefKey::Native(x) => Ok(Some(BinKeyDecoder::Native(x))),
                        EnumDefKey::Foreign(x) => Ok(Some(BinKeyDecoder::Foreign(x))),
                    }
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

    fn decode_map_end(&mut self, _map: Self::MapDecoder) -> anyhow::Result<()> {
        Ok(())
    }

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

    fn decode_entry_value(&mut self, _: Self::ValueDecoder) -> anyhow::Result<Self::AnyDecoder> {
        Ok(BinAnyDecoder::Read)
    }

    fn decode_enum_discriminant(
        &mut self,
        e: Self::DiscriminantDecoder,
    ) -> anyhow::Result<(Self::AnyDecoder, Self::VariantDecoder)> {
        Ok((
            match e.variant {
                EnumDefKey::Native(x) => BinAnyDecoder::U32(u32::try_from(x)?),
                EnumDefKey::Foreign(y) => BinAnyDecoder::Str(y),
            },
            (),
        ))
    }

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

    fn decode_enum_end(&mut self, _: Self::EnumCloser) -> anyhow::Result<()> {
        Ok(())
    }

    fn decode_some_inner(
        &mut self,
        _: Self::SomeDecoder,
    ) -> anyhow::Result<(Self::AnyDecoder, Self::SomeCloser)> {
        Ok((BinAnyDecoder::Read, ()))
    }

    fn decode_some_end(&mut self, _: Self::SomeCloser) -> anyhow::Result<()> {
        Ok(())
    }
}
