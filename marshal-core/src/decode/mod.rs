use std::borrow::Cow;
use std::error::Error;
use std::fmt::{Debug, Display, Formatter};
use std::marker::PhantomData;

use crate::{Primitive, PrimitiveType};

pub mod depth_budget;
mod helper;
pub mod newtype;
pub mod poison;
mod polonius;

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

pub struct SeqIter<'p, D: ?Sized + Decoder + 'p, F> {
    seq: SeqDecoder<'p, D>,
    map: F,
    phantom: PhantomData<&'p D>,
}

impl<'p, D: ?Sized + Decoder, T, F: for<'p2> FnMut(AnyDecoder<'p2, D>) -> anyhow::Result<T>>
    Iterator for SeqIter<'p, D, F>
{
    type Item = anyhow::Result<T>;

    fn next(&mut self) -> Option<Self::Item> {
        match self.seq.decode_next() {
            Err(e) => Some(Err(e)),
            Ok(None) => None,
            Ok(Some(x)) => Some((self.map)(x)),
        }
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        if let Some(size) = self.seq.exact_size() {
            (size, Some(size))
        } else {
            (0, None)
        }
    }
}

pub struct MapIter<'p, D: ?Sized + Decoder + 'p, C, KF, VF> {
    map: MapDecoder<'p, D>,
    ctx: C,
    key: KF,
    value: VF,
    phantom: PhantomData<&'p D>,
}

impl<
        'p,
        D: ?Sized + Decoder,
        C,
        K,
        V,
        KF: for<'p2> FnMut(&mut C, AnyDecoder<'p2, D>) -> anyhow::Result<K>,
        VF: for<'p2> FnMut(&mut C, K, AnyDecoder<'p2, D>) -> anyhow::Result<V>,
    > Iterator for MapIter<'p, D, C, KF, VF>
{
    type Item = anyhow::Result<V>;

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

pub enum SimpleDecoderView<P: ?Sized + Decoder> {
    Primitive(Primitive),
    String(P::StringDecoder),
    Bytes(P::BytesDecoder),
    None,
    Some(P::SomeDecoder),
    Seq(P::SeqDecoder),
    Map(P::MapDecoder),
    Enum(P::DiscriminantDecoder),
}

pub enum DecoderView<'p, P: ?Sized + Decoder>
where
    P: 'p,
{
    Primitive(Primitive),
    String(StringDecoder<'p, P>),
    Bytes(BytesDecoder<'p, P>),
    None,
    Some(SomeDecoder<'p, P>),
    Seq(SeqDecoder<'p, P>),
    Map(MapDecoder<'p, P>),
    Enum(EnumDecoder<'p, P>),
}

pub trait Decoder {
    type AnyDecoder;
    type StringDecoder;
    type BytesDecoder;
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
    ) -> anyhow::Result<SimpleDecoderView<Self>>;
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
    ) -> anyhow::Result<(SimpleDecoderView<Self>, Self::EnumCloser)>;

    fn decode_enum_end(&mut self, e: Self::EnumCloser) -> anyhow::Result<()>;

    fn decode_some_inner(
        &mut self,
        e: Self::SomeDecoder,
    ) -> anyhow::Result<(Self::AnyDecoder, Self::SomeCloser)>;

    fn decode_some_end(&mut self, p: Self::SomeCloser) -> anyhow::Result<()>;

    fn decode_string_cow(&mut self, p: Self::StringDecoder) -> anyhow::Result<Cow<str>>;

    fn decode_bytes_cow(&mut self, p: Self::BytesDecoder) -> anyhow::Result<Cow<[u8]>>;
}

pub struct AnyDecoder<'p, D: ?Sized + Decoder> {
    this: &'p mut D,
    any: D::AnyDecoder,
}

pub struct SeqDecoder<'p, D: ?Sized + Decoder> {
    this: &'p mut D,
    seq: Option<D::SeqDecoder>,
}

pub struct MapDecoder<'p, D: ?Sized + Decoder> {
    this: &'p mut D,
    map: Option<D::MapDecoder>,
}

pub struct StringDecoder<'p, D: ?Sized + Decoder> {
    this: &'p mut D,
    string: D::StringDecoder,
}

pub struct BytesDecoder<'p, D: ?Sized + Decoder> {
    this: &'p mut D,
    bytes: D::BytesDecoder,
}

pub struct EntryDecoder<'p, D: ?Sized + Decoder> {
    this: &'p mut D,
    key: Option<D::KeyDecoder>,
    value: Option<D::ValueDecoder>,
}

pub struct EnumDecoder<'p, D: ?Sized + Decoder> {
    this: &'p mut D,
    discriminant: Option<D::DiscriminantDecoder>,
    variant: Option<D::VariantDecoder>,
    closer: Option<D::EnumCloser>,
}

pub struct SomeDecoder<'p, D: ?Sized + Decoder> {
    this: &'p mut D,
    some_decoder: Option<D::SomeDecoder>,
    some_closer: Option<D::SomeCloser>,
}

impl<'p, D: ?Sized + Decoder> AnyDecoder<'p, D> {
    pub fn decode(self, hint: DecodeHint) -> anyhow::Result<DecoderView<'p, D>> {
        Ok(self.this.decode(self.any, hint)?.wrap(self.this))
    }
    pub fn ignore(self) -> anyhow::Result<()> {
        self.decode(DecodeHint::Ignore)?.ignore()
    }
}

impl<'p, D: ?Sized + Decoder> StringDecoder<'p, D> {
    pub fn decode_cow(self) -> anyhow::Result<Cow<'p, str>> {
        self.this.decode_string_cow(self.string)
    }
}

impl<'p, D: ?Sized + Decoder> BytesDecoder<'p, D> {
    pub fn decode_cow(self) -> anyhow::Result<Cow<'p, [u8]>> {
        self.this.decode_bytes_cow(self.bytes)
    }
}

impl<'p, D: ?Sized + Decoder> SeqDecoder<'p, D> {
    pub fn decode_next<'p2>(&'p2 mut self) -> anyhow::Result<Option<AnyDecoder<'p2, D>>> {
        if let Some(any) = self.this.decode_seq_next(self.seq.as_mut().unwrap())? {
            Ok(Some(AnyDecoder {
                this: self.this,
                any,
            }))
        } else {
            self.this.decode_seq_end(self.seq.take().unwrap())?;
            Ok(None)
        }
    }
    pub fn exact_size(&self) -> Option<usize> {
        self.this.decode_seq_exact_size(self.seq.as_ref().unwrap())
    }
    pub fn ignore(mut self) -> anyhow::Result<()> {
        while let Some(next) = self.decode_next()? {
            next.ignore()?;
        }
        Ok(())
    }
    pub fn seq_into_iter<T, F: for<'p2> FnMut(AnyDecoder<'p2, D>) -> anyhow::Result<T>>(
        self,
        map: F,
    ) -> SeqIter<'p, D, F> {
        SeqIter {
            seq: self,
            map,
            phantom: PhantomData,
        }
    }
}

impl<'p, D: ?Sized + Decoder> MapDecoder<'p, D> {
    pub fn decode_next<'p2>(&'p2 mut self) -> anyhow::Result<Option<EntryDecoder<'p2, D>>> {
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
    pub fn exact_size(&self) -> Option<usize> {
        self.this.decode_map_exact_size(self.map.as_ref().unwrap())
    }
    pub fn ignore(mut self) -> anyhow::Result<()> {
        while let Some(next) = self.decode_next()? {
            next.ignore()?;
        }
        Ok(())
    }
    pub fn map_into_iter<
        C,
        K,
        V,
        KF: for<'p2> FnMut(&mut C, AnyDecoder<'p2, D>) -> anyhow::Result<K>,
        VF: for<'p2> FnMut(&mut C, K, AnyDecoder<'p2, D>) -> anyhow::Result<V>,
    >(
        self,
        ctx: C,
        key: KF,
        value: VF,
    ) -> MapIter<'p, D, C, KF, VF> {
        MapIter {
            map: self,
            ctx,
            key,
            value,
            phantom: PhantomData,
        }
    }
}

impl<'p, D: ?Sized + Decoder> EntryDecoder<'p, D> {
    pub fn decode_key<'p2>(&'p2 mut self) -> anyhow::Result<AnyDecoder<'p2, D>> {
        let (key, value) = self.this.decode_entry_key(self.key.take().unwrap())?;
        self.value = Some(value);
        Ok(AnyDecoder {
            this: self.this,
            any: key,
        })
    }

    pub fn decode_value<'p2>(&'p2 mut self) -> anyhow::Result<AnyDecoder<'p2, D>> {
        let value = self.value.take().unwrap();
        let value = self.this.decode_entry_value(value)?;
        Ok(AnyDecoder {
            this: self.this,
            any: value,
        })
    }

    pub fn decode_end(mut self) -> anyhow::Result<()> {
        Ok(())
    }

    pub fn ignore(mut self) -> anyhow::Result<()> {
        self.decode_key()?.ignore()?;
        self.decode_value()?.ignore()?;
        self.decode_end()?;
        Ok(())
    }
}

impl<'p, T> EnumDecoder<'p, T>
where
    T: ?Sized + Decoder,
{
    pub fn decode_discriminant<'p2>(&'p2 mut self) -> anyhow::Result<AnyDecoder<'p2, T>> {
        let (discriminant, variant) = self
            .this
            .decode_enum_discriminant(self.discriminant.take().unwrap())?;
        self.variant = Some(variant);
        Ok(AnyDecoder {
            this: self.this,
            any: discriminant,
        })
    }

    pub fn decode_variant<'p2>(
        &'p2 mut self,
        hint: DecodeVariantHint,
    ) -> anyhow::Result<DecoderView<'p2, T>> {
        let (data, closer) = self
            .this
            .decode_enum_variant(self.variant.take().unwrap(), hint)?;
        self.closer = Some(closer);
        Ok(data.wrap(self.this))
    }

    pub fn decode_end(mut self) -> anyhow::Result<()> {
        self.this.decode_enum_end(self.closer.take().unwrap())
    }

    pub fn ignore(mut self) -> anyhow::Result<()> {
        self.decode_discriminant()?.ignore()?;
        self.decode_variant(DecodeVariantHint::Ignore)?.ignore()?;
        Ok(())
    }
}

impl<'p, D: ?Sized + Decoder> SomeDecoder<'p, D> {
    pub fn decode_some<'p2>(&'p2 mut self) -> anyhow::Result<AnyDecoder<'p2, D>> {
        let (any, closer) = self
            .this
            .decode_some_inner(self.some_decoder.take().unwrap())?;
        self.some_closer = Some(closer);
        Ok(AnyDecoder::new(self.this, any))
    }

    pub fn decode_end(mut self) -> anyhow::Result<()> {
        self.this.decode_some_end(self.some_closer.take().unwrap())
    }

    pub fn ignore(mut self) -> anyhow::Result<()> {
        self.decode_some()?.ignore()?;
        self.decode_end()?;
        Ok(())
    }
}

impl<'p, T: ?Sized + Decoder> AnyDecoder<'p, T> {
    pub fn new(decoder: &'p mut T, any: T::AnyDecoder) -> Self {
        AnyDecoder { this: decoder, any }
    }
}

impl<'p, D: ?Sized + Decoder> DecoderView<'p, D> {
    pub fn try_into_seq(self) -> anyhow::Result<SeqDecoder<'p, D>> {
        match self {
            DecoderView::Seq(x) => Ok(x),
            unexpected => unexpected.mismatch("seq")?,
        }
    }
    pub fn try_into_map(self) -> anyhow::Result<MapDecoder<'p, D>> {
        match self {
            DecoderView::Map(x) => Ok(x),
            unexpected => unexpected.mismatch("map")?,
        }
    }
    pub fn try_into_string(self) -> anyhow::Result<StringDecoder<'p, D>> {
        match self {
            DecoderView::String(x) => Ok(x),
            unexpected => unexpected.mismatch("string")?,
        }
    }
    pub fn try_into_option(self) -> anyhow::Result<Option<SomeDecoder<'p, D>>> {
        match self {
            DecoderView::None => Ok(None),
            DecoderView::Some(x) => Ok(Some(x)),
            unexpected => unexpected.mismatch("option")?,
        }
    }
    pub fn try_into_identifier(
        self,
        ids: &'static [&'static str],
    ) -> anyhow::Result<Option<usize>> {
        match self {
            DecoderView::Primitive(n) => Ok(Some(n.try_into()?)),
            DecoderView::String(s) => {
                let s = s.decode_cow()?;
                Ok(ids.iter().position(|x| **x == s))
            }
            unexpected => unexpected.mismatch("option")?,
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

impl<'p, P: Decoder> Debug for DecoderView<'p, P> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            DecoderView::Primitive(x) => f.debug_tuple("Primitive").field(x).finish(),
            DecoderView::String(x) => f.debug_struct("String").finish_non_exhaustive(),
            DecoderView::Bytes(x) => f.debug_struct("Bytes").finish_non_exhaustive(),
            DecoderView::None => f.debug_tuple("None").finish(),
            DecoderView::Some(_) => f.debug_struct("Some").finish_non_exhaustive(),
            DecoderView::Seq(_) => f.debug_struct("Seq").finish_non_exhaustive(),
            DecoderView::Map(_) => f.debug_struct("Map").finish_non_exhaustive(),
            DecoderView::Enum(_) => f.debug_struct("Enum").finish_non_exhaustive(),
        }
    }
}

impl<D: ?Sized + Decoder> SimpleDecoderView<D> {
    fn wrap<'p>(self, this: &'p mut D) -> DecoderView<'p, D> {
        match self {
            SimpleDecoderView::Primitive(x) => DecoderView::Primitive(x),
            SimpleDecoderView::String(string) => {
                DecoderView::String(StringDecoder { this, string })
            }
            SimpleDecoderView::Bytes(bytes) => {
                DecoderView::Bytes(BytesDecoder { this, bytes })
            },
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
