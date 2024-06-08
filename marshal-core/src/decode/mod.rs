use std::borrow::Cow;
use std::error::Error;
use std::fmt::{Debug, Display, Formatter};
use std::marker::PhantomData;

use crate::{Primitive, PrimitiveType};

pub mod depth_budget;
pub mod poison;
pub mod simple;

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

pub enum DecoderView<'p, 'de, P: ?Sized + Decoder<'de>>
where
    P: 'p,
{
    Primitive(Primitive),
    String(Cow<'de, str>),
    Bytes(Cow<'de, [u8]>),
    None,
    Some(P::SomeDecoder<'p>),
    Seq(P::SeqDecoder<'p>),
    Map(P::MapDecoder<'p>),
    Enum(P::EnumDecoder<'p>),
}

pub trait AnyDecoder<'p, 'de, P: ?Sized + Decoder<'de> + 'p>: Sized {
    fn decode(self, hint: DecodeHint) -> anyhow::Result<DecoderView<'p, 'de, P>>;
    fn ignore(mut self) -> anyhow::Result<()> {
        self.decode(DecodeHint::Ignore)?.ignore()
    }
}

pub trait SeqDecoder<'p, 'de, P: ?Sized + Decoder<'de>>: Sized {
    fn decode_next<'p2>(&'p2 mut self) -> anyhow::Result<Option<P::AnyDecoder<'p2>>>;
    fn exact_size(&self) -> Option<usize> {
        None
    }
    fn seq_into_iter<T, F: for<'p2> FnMut(P::AnyDecoder<'p2>) -> anyhow::Result<T>>(
        self,
        map: F,
    ) -> SeqIter<'p, 'de, Self, P, F> {
        SeqIter {
            seq: self,
            map,
            phantom: PhantomData,
        }
    }
    fn ignore(mut self) -> anyhow::Result<()> {
        while let Some(next) = self.decode_next()? {
            next.ignore()?;
        }
        Ok(())
    }
}

pub struct SeqIter<'p, 'de, S: SeqDecoder<'p, 'de, P>, P: Decoder<'de> + 'p, F> {
    seq: S,
    map: F,
    phantom: PhantomData<(&'p P, &'de ())>,
}

impl<
        'p,
        'de,
        S: SeqDecoder<'p, 'de, P>,
        P: Decoder<'de>,
        T,
        F: for<'p2> FnMut(P::AnyDecoder<'p2>) -> anyhow::Result<T>,
    > Iterator for SeqIter<'p, 'de, S, P, F>
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

pub trait MapDecoder<'p, 'de, P: ?Sized + Decoder<'de>>: Sized {
    fn decode_next<'p2>(&'p2 mut self) -> anyhow::Result<Option<P::EntryDecoder<'p2>>>;
    fn exact_size(&self) -> Option<usize> {
        None
    }
    fn map_into_iter<
        C,
        K,
        KF: for<'p2> FnMut(&mut C, P::AnyDecoder<'p2>) -> anyhow::Result<K>,
        V,
        VF: for<'p2> FnMut(&mut C, K, P::AnyDecoder<'p2>) -> anyhow::Result<V>,
    >(
        self,
        ctx: C,
        key: KF,
        value: VF,
    ) -> MapIter<'p, 'de, Self, P, C, KF, VF> {
        MapIter {
            map: self,
            ctx,
            key,
            value,
            phantom: PhantomData,
        }
    }
    fn ignore(mut self) -> anyhow::Result<()> {
        while let Some(next) = self.decode_next()? {
            next.ignore()?;
        }
        Ok(())
    }
}

pub struct MapIter<'p, 'de, M: MapDecoder<'p, 'de, P>, P: Decoder<'de> + 'p, C, KF, VF> {
    map: M,
    ctx: C,
    key: KF,
    value: VF,
    phantom: PhantomData<(&'p P, &'de ())>,
}

impl<
        'p,
        'de,
        M: MapDecoder<'p, 'de, P>,
        P: Decoder<'de>,
        C,
        K,
        V,
        KF: for<'p2> FnMut(&mut C, P::AnyDecoder<'p2>) -> anyhow::Result<K>,
        VF: for<'p2> FnMut(&mut C, K, P::AnyDecoder<'p2>) -> anyhow::Result<V>,
    > Iterator for MapIter<'p, 'de, M, P, C, KF, VF>
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

pub trait EntryDecoder<'p, 'de, P: ?Sized + Decoder<'de>>: Sized {
    fn decode_key<'p2>(&'p2 mut self) -> anyhow::Result<P::AnyDecoder<'p2>>;
    fn decode_value<'p2>(&'p2 mut self) -> anyhow::Result<P::AnyDecoder<'p2>>;
    fn decode_end(self) -> anyhow::Result<()>;
    fn ignore(mut self) -> anyhow::Result<()> {
        self.decode_key()?.ignore()?;
        self.decode_value()?.ignore()?;
        self.decode_end()
    }
}

pub trait EnumDecoder<'p, 'de, P: ?Sized + Decoder<'de>>: Sized {
    fn decode_discriminant<'p2>(&'p2 mut self) -> anyhow::Result<P::AnyDecoder<'p2>>;
    fn decode_variant<'p2>(
        &'p2 mut self,
        hint: DecodeVariantHint,
    ) -> anyhow::Result<DecoderView<'p2, 'de, P>>;
    fn decode_end(self) -> anyhow::Result<()>;
    fn ignore(mut self) -> anyhow::Result<()> {
        self.decode_discriminant()?.ignore()?;
        self.decode_variant(DecodeVariantHint::Ignore)?.ignore()?;
        self.decode_end()
    }
}

pub trait SomeDecoder<'p, 'de, P: ?Sized + Decoder<'de> + 'p>: Sized {
    fn decode_some<'p2>(&'p2 mut self) -> anyhow::Result<<P as Decoder<'de>>::AnyDecoder<'p2>>;
    fn decode_end(self) -> anyhow::Result<()>;
    fn ignore(mut self) -> anyhow::Result<()> {
        self.decode_some()?.ignore()?;
        self.decode_end()
    }
}
pub trait Decoder<'de>: Sized {
    type AnyDecoder<'p>: AnyDecoder<'p, 'de, Self>
    where
        Self: 'p;
    type SeqDecoder<'p>: SeqDecoder<'p, 'de, Self>
    where
        Self: 'p;
    type MapDecoder<'p>: MapDecoder<'p, 'de, Self>
    where
        Self: 'p;
    type EntryDecoder<'p>: EntryDecoder<'p, 'de, Self>
    where
        Self: 'p;
    type EnumDecoder<'p>: EnumDecoder<'p, 'de, Self>
    where
        Self: 'p;
    type SomeDecoder<'p>: SomeDecoder<'p, 'de, Self>
    where
        Self: 'p;
}

impl<'p, 'de, P: Decoder<'de>> Debug for DecoderView<'p, 'de, P> {
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

impl<'p, 'de, P: Decoder<'de>> DecoderView<'p, 'de, P> {
    pub fn try_into_seq(self) -> anyhow::Result<P::SeqDecoder<'p>> {
        match self {
            DecoderView::Seq(x) => Ok(x),
            unexpected => unexpected.mismatch("seq")?,
        }
    }
    pub fn try_into_map(self) -> anyhow::Result<P::MapDecoder<'p>> {
        match self {
            DecoderView::Map(x) => Ok(x),
            unexpected => unexpected.mismatch("map")?,
        }
    }
    pub fn try_into_string(self) -> anyhow::Result<Cow<'de, str>> {
        match self {
            DecoderView::String(x) => Ok(x),
            unexpected => unexpected.mismatch("string")?,
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
