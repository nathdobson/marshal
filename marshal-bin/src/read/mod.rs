pub mod full;

use by_address::ByAddress;
use num_traits::FromPrimitive;
use safe_once::cell::{OnceCell, RawFusedCell};
use safe_once_map::cell::OnceCellMap;
use std::borrow::Cow;
use std::cell::Cell;
use std::fmt::{Debug, Display, Formatter};

use marshal_core::parse::simple::{SimpleParser, SimpleParserView};
use marshal_core::parse::{ParseHint, ParseVariantHint};
use marshal_core::Primitive;

use crate::to_from_vu128::{Array, ToFromVu128};
use crate::util::StableCellVec;
use crate::{TypeTag, VU128_MAX_PADDING};
use safe_once_map::util::index_arena::IndexArena;
use safe_once_map::util::StableMap;

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

pub struct BinParserSchema {
    enum_defs: StableCellVec<EnumDefForeign>,
}

impl BinParserSchema {
    pub fn new() -> Self {
        BinParserSchema {
            enum_defs: StableCellVec::new(),
        }
    }
}

pub struct SimpleBinParser<'de, 's> {
    content: &'de [u8],
    schema: &'s BinParserSchema,
}

#[derive(Debug)]
pub enum BinParserError {
    TooMuchPadding,
    NonZeroPadding,
    Eof,
    BadTag,
    NoSuchEnumDef,
    NoMoreStructKeys,
}

impl Display for BinParserError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        Debug::fmt(self, f)
    }
}

impl std::error::Error for BinParserError {}

impl<'de, 's> SimpleBinParser<'de, 's> {
    pub fn new(data: &'de [u8], schema: &'s mut BinParserSchema) -> SimpleBinParser<'de, 's> {
        SimpleBinParser {
            content: data,
            schema,
        }
    }
    pub fn end(self) -> anyhow::Result<()> {
        if self.content.len() > VU128_MAX_PADDING {
            return Err(BinParserError::TooMuchPadding.into());
        }
        if self.content.iter().any(|x| *x != 0) {
            return Err(BinParserError::NonZeroPadding.into());
        }
        Ok(())
    }
}

impl<'de, 's> SimpleBinParser<'de, 's> {
    pub fn read_count(&mut self, count: usize) -> anyhow::Result<&'de [u8]> {
        Ok(self.content.take(..count).ok_or(BinParserError::Eof)?)
    }
    pub fn read_vu128<T: ToFromVu128 + Display>(&mut self) -> anyhow::Result<T> {
        let (value, count) = T::decode_vu128(T::Buffer::try_from_slice(
            &self.content[..T::Buffer::ARRAY_LEN],
        )?);
        self.content.take(..count).ok_or(BinParserError::Eof)?;
        Ok(value)
    }
    pub fn read_usize(&mut self) -> anyhow::Result<usize> {
        Ok(usize::try_from(self.read_vu128::<u64>()?)?)
    }
    pub fn parse_type_tag(&mut self) -> anyhow::Result<TypeTag> {
        Ok(TypeTag::from_u8(self.read_count(1)?[0]).ok_or(BinParserError::BadTag)?)
    }
    pub fn read_bytes(&mut self) -> anyhow::Result<&'de [u8]> {
        let len = self.read_usize()?;
        self.read_count(len)
    }
    pub fn read_str(&mut self) -> anyhow::Result<&'de str> {
        Ok(std::str::from_utf8(self.read_bytes()?)?)
    }
    pub fn read_enum_def(&mut self) -> anyhow::Result<()> {
        let count = self.read_usize()?;
        let fields = (0..count)
            .map(|x| Ok(self.read_str()?.to_string()))
            .collect::<anyhow::Result<Vec<_>>>()?;
        let def = EnumDefForeign {
            fields,
            default_translation: EnumDefTranslation { keys: vec![] },
            custom_translation: Default::default(),
        };
        let index = self.schema.enum_defs.push(def);
        println!("at index {}", index);
        Ok(())
    }
    pub fn read_enum_def_ref(&mut self) -> anyhow::Result<&'s EnumDefForeign> {
        let index = self.read_usize()?;
        Ok(self
            .schema
            .enum_defs
            .get(index)
            .ok_or(BinParserError::NoSuchEnumDef)?)
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

pub enum BinAnyParser<'s> {
    U32(u32),
    Str(&'s str),
    Read,
}

pub struct BinSeqParser {
    len: usize,
}

pub struct BinMapParser<'s> {
    schema: Option<&'s [EnumDefKey]>,
}

pub enum BinKeyParser<'s> {
    Foreign(&'s str),
    Native(usize),
    Raw,
}

impl<'de, 's> SimpleParser<'de> for SimpleBinParser<'de, 's> {
    type AnyParser = BinAnyParser<'s>;
    type SeqParser = BinSeqParser;
    type MapParser = BinMapParser<'s>;
    type KeyParser = BinKeyParser<'s>;
    type ValueParser = ();
    type DiscriminantParser = ();
    type VariantParser = ();
    type SomeParser = ();

    fn parse(
        &mut self,
        any: Self::AnyParser,
        hint: ParseHint,
    ) -> anyhow::Result<SimpleParserView<'de, Self>> {
        match any{
            BinAnyParser::U32(x) => return Ok(SimpleParserView::Primitive(Primitive::U32(x))),
            BinAnyParser::Str(x) => return Ok(SimpleParserView::String(Cow::Owned(x.to_string()))),
            BinAnyParser::Read => {}
        }
        loop {
            let tag = self.parse_type_tag()?;
            let () = match tag {
                TypeTag::Unit => return Ok(SimpleParserView::Primitive(Primitive::Unit)),
                TypeTag::Bool => {
                    return Ok(SimpleParserView::Primitive(Primitive::Bool(
                        self.read_vu128()?,
                    )))
                }
                TypeTag::I8 => {
                    return Ok(SimpleParserView::Primitive(Primitive::I8(
                        self.read_vu128()?,
                    )))
                }
                TypeTag::I16 => {
                    return Ok(SimpleParserView::Primitive(Primitive::I16(
                        self.read_vu128()?,
                    )))
                }
                TypeTag::I32 => {
                    return Ok(SimpleParserView::Primitive(Primitive::I32(
                        self.read_vu128()?,
                    )))
                }
                TypeTag::I64 => {
                    return Ok(SimpleParserView::Primitive(Primitive::I64(
                        self.read_vu128()?,
                    )))
                }
                TypeTag::I128 => {
                    return Ok(SimpleParserView::Primitive(Primitive::I128(
                        self.read_vu128()?,
                    )))
                }
                TypeTag::U8 => {
                    return Ok(SimpleParserView::Primitive(Primitive::U8(
                        self.read_vu128()?,
                    )))
                }
                TypeTag::U16 => {
                    return Ok(SimpleParserView::Primitive(Primitive::U16(
                        self.read_vu128()?,
                    )))
                }
                TypeTag::U32 => {
                    return Ok(SimpleParserView::Primitive(Primitive::U32(
                        self.read_vu128()?,
                    )))
                }
                TypeTag::U64 => {
                    return Ok(SimpleParserView::Primitive(Primitive::U64(
                        self.read_vu128()?,
                    )))
                }
                TypeTag::U128 => {
                    return Ok(SimpleParserView::Primitive(Primitive::U128(
                        self.read_vu128()?,
                    )))
                }
                TypeTag::F32 => {
                    return Ok(SimpleParserView::Primitive(Primitive::F32(
                        self.read_vu128()?,
                    )))
                }
                TypeTag::F64 => {
                    return Ok(SimpleParserView::Primitive(Primitive::F32(
                        self.read_vu128()?,
                    )))
                }
                TypeTag::Char => {
                    return Ok(SimpleParserView::Primitive(Primitive::Char(
                        self.read_vu128::<u32>()?.try_into()?,
                    )))
                }
                TypeTag::Struct => {
                    let enum_def = self.read_enum_def_ref()?;
                    let fields = match hint {
                        ParseHint::Struct { name, fields } => Some(fields),
                        _ => None,
                    };
                    let trans = enum_def.get_translation(fields);
                    return Ok(SimpleParserView::Map(BinMapParser {
                        schema: Some(&trans.keys),
                    }));
                }
                TypeTag::TupleStruct => todo!(),
                TypeTag::Enum => todo!(),
                TypeTag::Seq => todo!(),
                TypeTag::Map => todo!(),
                TypeTag::Tuple => {
                    let len = self.read_vu128::<u64>()?;
                    return Ok(SimpleParserView::Seq(BinSeqParser {
                        len: usize::try_from(len)?,
                    }));
                }
                TypeTag::EnumDef => self.read_enum_def()?,
                TypeTag::String => return Ok(SimpleParserView::String(self.read_str()?.into())),
            };
        }
    }

    fn is_human_readable(&self) -> bool {
        todo!()
    }

    fn parse_seq_next(
        &mut self,
        seq: &mut Self::SeqParser,
    ) -> anyhow::Result<Option<Self::AnyParser>> {
        if let Some(len2) = seq.len.checked_sub(1) {
            seq.len = len2;
            Ok(Some(BinAnyParser::Read))
        } else {
            Ok(None)
        }
    }

    fn parse_map_next(
        &mut self,
        map: &mut Self::MapParser,
    ) -> anyhow::Result<Option<Self::KeyParser>> {
        if let Some(schema) = &mut map.schema {
            if let Some(key) = schema.take_first() {
                match key {
                    EnumDefKey::Native(x) => Ok(Some(BinKeyParser::Native(*x))),
                    EnumDefKey::Foreign(x) => Ok(Some(BinKeyParser::Foreign(x))),
                }
            } else {
                Ok(None)
            }
        } else {
            todo!()
        }
    }

    fn parse_entry_key(
        &mut self,
        key: Self::KeyParser,
    ) -> anyhow::Result<(Self::AnyParser, Self::ValueParser)> {
        match key {
            BinKeyParser::Foreign(x) => Ok((BinAnyParser::Str(x), ())),
            BinKeyParser::Native(x) => Ok((BinAnyParser::U32(u32::try_from(x)?), ())),
            BinKeyParser::Raw => Ok((BinAnyParser::Read, ())),
        }
    }

    fn parse_entry_value(&mut self, value: Self::ValueParser) -> anyhow::Result<Self::AnyParser> {
        Ok(BinAnyParser::Read)
    }

    fn parse_enum_discriminant(
        &mut self,
        e: Self::DiscriminantParser,
    ) -> anyhow::Result<(Self::AnyParser, Self::VariantParser)> {
        todo!()
    }

    fn parse_enum_variant(
        &mut self,
        e: Self::VariantParser,
        hint: ParseVariantHint,
    ) -> anyhow::Result<SimpleParserView<'de, Self>> {
        todo!()
    }
}
