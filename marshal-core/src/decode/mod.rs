use std::borrow::Cow;
use std::error::Error;
use std::fmt::{Debug, Display, Formatter};
use std::marker::PhantomData;

use crate::{Primitive, PrimitiveType, SchemaError};

pub mod depth_budget;
mod enum_helper;
pub mod newtype;
pub mod poison;
mod struct_helper;
mod tuple_helper;
// pub mod depth_budget;
// pub mod poison;

#[derive(Debug, Copy, Clone)]
pub enum DecodeHint {
    Any,
    Primitive(PrimitiveType),
    String,
    Bytes,
    Option,
    UnitStruct {
        name: &'static str,
    },
    Seq,
    Tuple {
        len: usize,
    },
    TupleStruct {
        name: &'static str,
        len: usize,
    },
    Map,
    Struct {
        name: &'static str,
        fields: &'static [&'static str],
    },
    Enum {
        name: &'static str,
        variants: &'static [&'static str],
    },
    Identifier,
    Ignore,
}

pub enum DecodeVariantHint {
    UnitVariant,
    TupleVariant { len: usize },
    StructVariant { fields: &'static [&'static str] },
    Ignore,
}

pub struct SeqIter<'p, 'de, D: ?Sized + SpecDecoder<'de> + 'p, F> {
    seq: SeqDecoder<'p, 'de, D>,
    map: F,
    phantom: PhantomData<(&'p D, &'de ())>,
}

impl<
        'p,
        'de,
        D: ?Sized + SpecDecoder<'de>,
        T,
        F: for<'p2> FnMut(AnySpecDecoder<'p2, 'de, D>) -> anyhow::Result<T>,
    > Iterator for SeqIter<'p, 'de, D, F>
{
    type Item = anyhow::Result<T>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        match self.seq.decode_next() {
            Err(e) => Some(Err(e)),
            Ok(None) => None,
            Ok(Some(x)) => Some((self.map)(x)),
        }
    }
    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        if let Some(size) = self.seq.exact_size() {
            (size, Some(size))
        } else {
            (0, None)
        }
    }
}

pub struct MapIter<'p, 'de, D: ?Sized + SpecDecoder<'de> + 'p, C, KF, VF> {
    map: MapDecoder<'p, 'de, D>,
    ctx: C,
    key: KF,
    value: VF,
    phantom: PhantomData<(&'p D, &'de ())>,
}

impl<
        'p,
        'de,
        D: ?Sized + SpecDecoder<'de>,
        C,
        K,
        V,
        KF: for<'p2> FnMut(&mut C, AnySpecDecoder<'p2, 'de, D>) -> anyhow::Result<K>,
        VF: for<'p2> FnMut(&mut C, K, AnySpecDecoder<'p2, 'de, D>) -> anyhow::Result<V>,
    > Iterator for MapIter<'p, 'de, D, C, KF, VF>
{
    type Item = anyhow::Result<V>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let result: anyhow::Result<Option<V>> = try {
            match self.map.decode_next()? {
                None => None,
                Some(mut p) => {
                    let key = (self.key)(&mut self.ctx, p.decode_key()?)?;
                    let value = (self.value)(&mut self.ctx, key, p.decode_value()?)?;
                    p.decode_end()?;
                    Some(value)
                }
            }
        };
        result.transpose()
    }
    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        if let Some(size) = self.map.exact_size() {
            (size, Some(size))
        } else {
            (0, None)
        }
    }
}

#[derive(Debug)]
pub struct TypeMismatch {
    pub found: &'static str,
    pub expected: &'static str,
}

impl Display for TypeMismatch {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "found type `{}', expected type `{}'",
            self.found, self.expected
        )
    }
}

impl Error for TypeMismatch {}

impl Primitive {
    pub fn kind(&self) -> &'static str {
        match self {
            Primitive::Unit => "unit",
            Primitive::Bool(_) => "bool",
            Primitive::I8(_) => "i8",
            Primitive::I16(_) => "i16",
            Primitive::I32(_) => "i32",
            Primitive::I64(_) => "i64",
            Primitive::I128(_) => "i128",
            Primitive::U8(_) => "u8",
            Primitive::U16(_) => "u16",
            Primitive::U32(_) => "u32",
            Primitive::U64(_) => "u64",
            Primitive::U128(_) => "u128",
            Primitive::F32(_) => "f32",
            Primitive::F64(_) => "f64",
            Primitive::Char(_) => "char",
        }
    }
    pub fn mismatch(&self, expected: &'static str) -> anyhow::Result<!> {
        Err(TypeMismatch {
            found: self.kind(),
            expected,
        }
        .into())
    }
}

pub enum SimpleDecoderView<'de, P: ?Sized + SpecDecoder<'de>> {
    Primitive(Primitive),
    String(Cow<'de, str>),
    Bytes(Cow<'de, [u8]>),
    None,
    Some(P::SomeDecoder),
    Seq(P::SeqDecoder),
    Map(P::MapDecoder),
    Enum(P::DiscriminantDecoder),
}

pub enum DecoderView<'p, 'de, P: ?Sized + SpecDecoder<'de>>
where
    P: 'p,
{
    Primitive(Primitive),
    String(Cow<'de, str>),
    Bytes(Cow<'de, [u8]>),
    None,
    Some(SomeDecoder<'p, 'de, P>),
    Seq(SeqDecoder<'p, 'de, P>),
    Map(MapDecoder<'p, 'de, P>),
    Enum(EnumDecoder<'p, 'de, P>),
}

pub trait Decoder: 'static {
    type SpecDecoder<'de>: ?Sized + SpecDecoder<'de>;
}

pub trait SpecDecoder<'de> {
    type AnyDecoder;
    type SeqDecoder;
    type MapDecoder;
    type KeyDecoder;
    type ValueDecoder;
    type DiscriminantDecoder;
    type VariantDecoder;
    type EnumCloser;
    type SomeDecoder;
    type SomeCloser;

    fn decode(
        &mut self,
        any: Self::AnyDecoder,
        hint: DecodeHint,
    ) -> anyhow::Result<SimpleDecoderView<'de, Self>>;
    fn is_human_readable(&self) -> bool;

    fn decode_seq_next(
        &mut self,
        seq: &mut Self::SeqDecoder,
    ) -> anyhow::Result<Option<Self::AnyDecoder>>;
    fn decode_seq_exact_size(&self, _seq: &Self::SeqDecoder) -> Option<usize> {
        None
    }
    fn decode_seq_end(&mut self, seq: Self::SeqDecoder) -> anyhow::Result<()>;

    fn decode_map_next(
        &mut self,
        map: &mut Self::MapDecoder,
    ) -> anyhow::Result<Option<Self::KeyDecoder>>;
    fn decode_map_exact_size(&self, _map: &Self::MapDecoder) -> Option<usize> {
        None
    }
    fn decode_map_end(&mut self, seq: Self::MapDecoder) -> anyhow::Result<()>;

    fn decode_entry_key(
        &mut self,
        key: Self::KeyDecoder,
    ) -> anyhow::Result<(Self::AnyDecoder, Self::ValueDecoder)>;

    fn decode_entry_value(&mut self, value: Self::ValueDecoder)
        -> anyhow::Result<Self::AnyDecoder>;

    fn decode_enum_discriminant(
        &mut self,
        e: Self::DiscriminantDecoder,
    ) -> anyhow::Result<(Self::AnyDecoder, Self::VariantDecoder)>;

    fn decode_enum_variant(
        &mut self,
        e: Self::VariantDecoder,
        hint: DecodeVariantHint,
    ) -> anyhow::Result<(SimpleDecoderView<'de, Self>, Self::EnumCloser)>;

    fn decode_enum_end(&mut self, e: Self::EnumCloser) -> anyhow::Result<()>;

    fn decode_some_inner(
        &mut self,
        e: Self::SomeDecoder,
    ) -> anyhow::Result<(Self::AnyDecoder, Self::SomeCloser)>;

    fn decode_some_end(&mut self, p: Self::SomeCloser) -> anyhow::Result<()>;
}

pub struct AnySpecDecoder<'p, 'de, D: ?Sized + SpecDecoder<'de>> {
    this: &'p mut D,
    any: D::AnyDecoder,
}

pub type AnyDecoder<'p, 'de, D> = AnySpecDecoder<'p, 'de, <D as Decoder>::SpecDecoder<'de>>;

pub struct SeqDecoder<'p, 'de, D: ?Sized + SpecDecoder<'de>> {
    this: &'p mut D,
    seq: Option<D::SeqDecoder>,
}

pub struct MapDecoder<'p, 'de, D: ?Sized + SpecDecoder<'de>> {
    this: &'p mut D,
    map: Option<D::MapDecoder>,
}

pub struct EntryDecoder<'p, 'de, D: ?Sized + SpecDecoder<'de>> {
    this: &'p mut D,
    key: Option<D::KeyDecoder>,
    value: Option<D::ValueDecoder>,
}

pub struct EnumDecoder<'p, 'de, D: ?Sized + SpecDecoder<'de>> {
    this: &'p mut D,
    discriminant: Option<D::DiscriminantDecoder>,
    variant: Option<D::VariantDecoder>,
    closer: Option<D::EnumCloser>,
}

pub struct SomeDecoder<'p, 'de, D: ?Sized + SpecDecoder<'de>> {
    this: &'p mut D,
    some_decoder: Option<D::SomeDecoder>,
    some_closer: Option<D::SomeCloser>,
}

impl<'p, 'de, D: ?Sized + SpecDecoder<'de>> MapDecoder<'p, 'de, D> {
    #[inline]
    pub fn into_raw(self) -> (&'p mut D, D::MapDecoder) {
        (self.this, self.map.unwrap())
    }
}

impl<'p, 'de, D: ?Sized + SpecDecoder<'de>> AnySpecDecoder<'p, 'de, D> {
    #[inline]
    pub fn decode(self, hint: DecodeHint) -> anyhow::Result<DecoderView<'p, 'de, D>> {
        Ok(self.this.decode(self.any, hint)?.wrap(self.this))
    }
    #[inline]
    pub fn ignore(self) -> anyhow::Result<()> {
        self.decode(DecodeHint::Ignore)?.ignore()
    }
    #[inline]
    pub fn is_human_readable(&self) -> bool {
        self.this.is_human_readable()
    }
}

impl<'p, 'de, D: ?Sized + SpecDecoder<'de>> SeqDecoder<'p, 'de, D> {
    #[inline]
    pub fn decode_next<'p2>(&'p2 mut self) -> anyhow::Result<Option<AnySpecDecoder<'p2, 'de, D>>> {
        if let Some(seq) = &mut self.seq {
            if let Some(any) = self.this.decode_seq_next(seq)? {
                Ok(Some(AnySpecDecoder {
                    this: self.this,
                    any,
                }))
            } else {
                self.this.decode_seq_end(self.seq.take().unwrap())?;
                Ok(None)
            }
        } else {
            Ok(None)
        }
    }
    #[inline]
    pub fn exact_size(&self) -> Option<usize> {
        self.this.decode_seq_exact_size(self.seq.as_ref().unwrap())
    }
    #[inline]
    pub fn ignore(mut self) -> anyhow::Result<()> {
        while let Some(next) = self.decode_next()? {
            next.ignore()?;
        }
        Ok(())
    }
    #[inline]
    pub fn seq_into_iter<T, F: for<'p2> FnMut(AnySpecDecoder<'p2, 'de, D>) -> anyhow::Result<T>>(
        self,
        map: F,
    ) -> SeqIter<'p, 'de, D, F> {
        SeqIter {
            seq: self,
            map,
            phantom: PhantomData,
        }
    }
}

impl<'p, 'de, D: ?Sized + SpecDecoder<'de>> MapDecoder<'p, 'de, D> {
    #[inline]
    pub fn decode_next<'p2>(&'p2 mut self) -> anyhow::Result<Option<EntryDecoder<'p2, 'de, D>>> {
        if let Some(data) = self.this.decode_map_next(self.map.as_mut().unwrap())? {
            Ok(Some(EntryDecoder {
                this: self.this,
                key: Some(data),
                value: None,
            }))
        } else {
            self.this.decode_map_end(self.map.take().unwrap())?;
            Ok(None)
        }
    }
    #[inline]
    pub fn exact_size(&self) -> Option<usize> {
        self.this.decode_map_exact_size(self.map.as_ref().unwrap())
    }
    #[inline]
    pub fn ignore(mut self) -> anyhow::Result<()> {
        while let Some(next) = self.decode_next()? {
            next.ignore()?;
        }
        Ok(())
    }
    #[inline]
    pub fn map_into_iter<
        C,
        K,
        V,
        KF: for<'p2> FnMut(&mut C, AnySpecDecoder<'p2, 'de, D>) -> anyhow::Result<K>,
        VF: for<'p2> FnMut(&mut C, K, AnySpecDecoder<'p2, 'de, D>) -> anyhow::Result<V>,
    >(
        self,
        ctx: C,
        key: KF,
        value: VF,
    ) -> MapIter<'p, 'de, D, C, KF, VF> {
        MapIter {
            map: self,
            ctx,
            key,
            value,
            phantom: PhantomData,
        }
    }
}

impl<'p, 'de, D: ?Sized + SpecDecoder<'de>> EntryDecoder<'p, 'de, D> {
    #[inline]
    pub fn decode_key<'p2>(&'p2 mut self) -> anyhow::Result<AnySpecDecoder<'p2, 'de, D>> {
        let (key, value) = self.this.decode_entry_key(self.key.take().unwrap())?;
        self.value = Some(value);
        Ok(AnySpecDecoder {
            this: self.this,
            any: key,
        })
    }

    #[inline]
    pub fn decode_value<'p2>(&'p2 mut self) -> anyhow::Result<AnySpecDecoder<'p2, 'de, D>> {
        let value = self.value.take().unwrap();
        let value = self.this.decode_entry_value(value)?;
        Ok(AnySpecDecoder {
            this: self.this,
            any: value,
        })
    }

    #[inline]
    pub fn decode_end(mut self) -> anyhow::Result<()> {
        Ok(())
    }

    #[inline]
    pub fn ignore(mut self) -> anyhow::Result<()> {
        self.decode_key()?.ignore()?;
        self.decode_value()?.ignore()?;
        self.decode_end()?;
        Ok(())
    }
}

impl<'p, 'de, T> EnumDecoder<'p, 'de, T>
where
    T: ?Sized + SpecDecoder<'de>,
{
    #[inline]
    pub fn decode_discriminant<'p2>(&'p2 mut self) -> anyhow::Result<AnySpecDecoder<'p2, 'de, T>> {
        let (discriminant, variant) = self
            .this
            .decode_enum_discriminant(self.discriminant.take().unwrap())?;
        self.variant = Some(variant);
        Ok(AnySpecDecoder {
            this: self.this,
            any: discriminant,
        })
    }

    #[inline]
    pub fn decode_variant<'p2>(
        &'p2 mut self,
        hint: DecodeVariantHint,
    ) -> anyhow::Result<DecoderView<'p2, 'de, T>> {
        let (data, closer) = self
            .this
            .decode_enum_variant(self.variant.take().unwrap(), hint)?;
        self.closer = Some(closer);
        Ok(data.wrap(self.this))
    }

    #[inline]
    pub fn decode_end(mut self) -> anyhow::Result<()> {
        self.this.decode_enum_end(self.closer.take().unwrap())
    }

    #[inline]
    pub fn ignore(mut self) -> anyhow::Result<()> {
        self.decode_discriminant()?.ignore()?;
        self.decode_variant(DecodeVariantHint::Ignore)?.ignore()?;
        Ok(())
    }
}

impl<'p, 'de, D: ?Sized + SpecDecoder<'de>> SomeDecoder<'p, 'de, D> {
    #[inline]
    pub fn decode_some<'p2>(&'p2 mut self) -> anyhow::Result<AnySpecDecoder<'p2, 'de, D>> {
        let (any, closer) = self
            .this
            .decode_some_inner(self.some_decoder.take().unwrap())?;
        self.some_closer = Some(closer);
        Ok(AnySpecDecoder::new(self.this, any))
    }

    #[inline]
    pub fn decode_end(mut self) -> anyhow::Result<()> {
        self.this.decode_some_end(self.some_closer.take().unwrap())
    }

    #[inline]
    pub fn ignore(mut self) -> anyhow::Result<()> {
        self.decode_some()?.ignore()?;
        self.decode_end()?;
        Ok(())
    }
}

impl<'p, 'de, T: ?Sized + SpecDecoder<'de>> AnySpecDecoder<'p, 'de, T> {
    #[inline]
    pub fn new(decoder: &'p mut T, any: T::AnyDecoder) -> Self {
        AnySpecDecoder { this: decoder, any }
    }
}

impl<'p, 'de, T: ?Sized + SpecDecoder<'de>> EntryDecoder<'p, 'de, T> {
    #[inline]
    pub fn new(decoder: &'p mut T, key: T::KeyDecoder) -> Self {
        EntryDecoder {
            this: decoder,
            key: Some(key),
            value: None,
        }
    }
}

impl<'p, 'de, D: ?Sized + SpecDecoder<'de>> DecoderView<'p, 'de, D> {
    #[inline]
    pub fn try_into_seq(self) -> anyhow::Result<SeqDecoder<'p, 'de, D>> {
        match self {
            DecoderView::Seq(x) => Ok(x),
            unexpected => unexpected.mismatch("seq")?,
        }
    }
    #[inline]
    pub fn try_into_map(self) -> anyhow::Result<MapDecoder<'p, 'de, D>> {
        match self {
            DecoderView::Map(x) => Ok(x),
            unexpected => unexpected.mismatch("map")?,
        }
    }
    #[inline]
    pub fn try_into_string(self) -> anyhow::Result<Cow<'de, str>> {
        match self {
            DecoderView::String(x) => Ok(x),
            unexpected => unexpected.mismatch("string")?,
        }
    }
    #[inline]
    pub fn try_into_option(self) -> anyhow::Result<Option<SomeDecoder<'p, 'de, D>>> {
        match self {
            DecoderView::None => Ok(None),
            DecoderView::Some(x) => Ok(Some(x)),
            unexpected => unexpected.mismatch("option")?,
        }
    }
    #[inline]
    pub fn try_into_enum(self) -> anyhow::Result<EnumDecoder<'p, 'de, D>> {
        match self {
            DecoderView::Enum(e) => Ok(e),
            unexpected => unexpected.mismatch("enum")?,
        }
    }
    #[inline]
    pub fn try_into_identifier(
        self,
        ids: &'static [&'static str],
    ) -> anyhow::Result<Result<usize, Cow<'de, str>>> {
        match self {
            DecoderView::Primitive(n) => Ok(Ok(n.try_into()?)),
            DecoderView::String(s) => Ok(ids.iter().position(|x| **x == s).ok_or(s)),
            unexpected => unexpected.mismatch("identifier")?,
        }
    }
    #[inline]
    pub fn try_into_discriminant(self, ids: &'static [&'static str]) -> anyhow::Result<usize> {
        Ok(self
            .try_into_identifier(ids)?
            .map_err(|v| SchemaError::UnknownDiscriminantName {
                disc: v.into_owned(),
                expected: ids,
            })?)
    }
    #[inline]
    pub fn try_into_unit(self) -> anyhow::Result<()> {
        match self {
            DecoderView::Primitive(p) => p.try_into(),
            unexpected => unexpected.mismatch("unit")?,
        }
    }
    pub fn kind(&self) -> &'static str {
        match self {
            DecoderView::Primitive(p) => p.kind(),
            DecoderView::String(_) => "string",
            DecoderView::Bytes(_) => "bytes",
            DecoderView::None => "none",
            DecoderView::Some(_) => "some",
            DecoderView::Seq(_) => "seq",
            DecoderView::Map(_) => "map",
            DecoderView::Enum(_) => "enum",
        }
    }
    pub fn mismatch(&self, expected: &'static str) -> anyhow::Result<!> {
        Err(TypeMismatch {
            found: self.kind(),
            expected,
        }
        .into())
    }
    #[inline]
    pub fn ignore(self) -> anyhow::Result<()> {
        match self {
            DecoderView::Primitive(_) => {}
            DecoderView::String(_) => {}
            DecoderView::Bytes(_) => {}
            DecoderView::None => {}
            DecoderView::Some(x) => x.ignore()?,
            DecoderView::Seq(x) => x.ignore()?,
            DecoderView::Map(x) => x.ignore()?,
            DecoderView::Enum(x) => x.ignore()?,
        }
        Ok(())
    }
}

impl<'p, 'de, P: SpecDecoder<'de>> Debug for DecoderView<'p, 'de, P> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            DecoderView::Primitive(x) => f.debug_tuple("Primitive").field(x).finish(),
            DecoderView::String(x) => f.debug_tuple("String").field(x).finish(),
            DecoderView::Bytes(x) => f.debug_tuple("Bytes").field(x).finish(),
            DecoderView::None => f.debug_tuple("None").finish(),
            DecoderView::Some(_) => f.debug_struct("Some").finish_non_exhaustive(),
            DecoderView::Seq(_) => f.debug_struct("Seq").finish_non_exhaustive(),
            DecoderView::Map(_) => f.debug_struct("Map").finish_non_exhaustive(),
            DecoderView::Enum(_) => f.debug_struct("Enum").finish_non_exhaustive(),
        }
    }
}

impl<'de, D: ?Sized + SpecDecoder<'de>> SimpleDecoderView<'de, D> {
    #[inline]
    fn wrap<'p>(self, this: &'p mut D) -> DecoderView<'p, 'de, D> {
        match self {
            SimpleDecoderView::Primitive(x) => DecoderView::Primitive(x),
            SimpleDecoderView::String(x) => DecoderView::String(x),
            SimpleDecoderView::Bytes(x) => DecoderView::Bytes(x),
            SimpleDecoderView::None => DecoderView::None,
            SimpleDecoderView::Some(some) => DecoderView::Some(SomeDecoder {
                this,
                some_decoder: Some(some),
                some_closer: None,
            }),
            SimpleDecoderView::Seq(seq) => DecoderView::Seq(SeqDecoder {
                this,
                seq: Some(seq),
            }),
            SimpleDecoderView::Map(map) => DecoderView::Map(MapDecoder {
                this,
                map: Some(map),
            }),
            SimpleDecoderView::Enum(data) => DecoderView::Enum(EnumDecoder {
                this,
                discriminant: Some(data),
                variant: None,
                closer: None,
            }),
        }
    }
}
