use std::borrow::Cow;
use std::error::Error;
use std::fmt::{Debug, Display, Formatter};
use std::marker::PhantomData;
use crate::{Primitive, PrimitiveType};


pub mod depth_budget;
pub mod poison;
pub mod simple;

#[derive(Debug, Copy, Clone)]
pub enum ParseHint {
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
}

pub enum ParseVariantHint {
    UnitVariant,
    TupleVariant,
    StructVariant,
}

pub enum ParserView<'p, 'de, P: ?Sized + Parser<'de>>
where
    P: 'p,
{
    Primitive(Primitive),
    String(Cow<'de, str>),
    Bytes(Cow<'de, [u8]>),
    None,
    Some(P::SomeParser<'p>),
    Seq(P::SeqParser<'p>),
    Map(P::MapParser<'p>),
    Enum(P::EnumParser<'p>),
}

pub trait AnyParser<'p, 'de, P: ?Sized + Parser<'de>> {
    fn parse(self, hint: ParseHint) -> anyhow::Result<ParserView<'p, 'de, P>>;
}

pub trait SeqParser<'p, 'de, P: ?Sized + Parser<'de>>: Sized {
    fn parse_next<'p2>(&'p2 mut self) -> anyhow::Result<Option<P::AnyParser<'p2>>>;
    fn exact_size(&self) -> Option<usize> {
        None
    }
    fn seq_into_iter<T, F: for<'p2> FnMut(P::AnyParser<'p2>) -> anyhow::Result<T>>(
        self,
        map: F,
    ) -> SeqIter<'p, 'de, Self, P, F> {
        SeqIter {
            seq: self,
            map,
            phantom: PhantomData,
        }
    }
}

pub struct SeqIter<'p, 'de, S: SeqParser<'p, 'de, P>, P: Parser<'de> + 'p, F> {
    seq: S,
    map: F,
    phantom: PhantomData<(&'p P, &'de ())>,
}

impl<
        'p,
        'de,
        S: SeqParser<'p, 'de, P>,
        P: Parser<'de>,
        T,
        F: for<'p2> FnMut(P::AnyParser<'p2>) -> anyhow::Result<T>,
    > Iterator for SeqIter<'p, 'de, S, P, F>
{
    type Item = anyhow::Result<T>;

    fn next(&mut self) -> Option<Self::Item> {
        match self.seq.parse_next() {
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

pub trait MapParser<'p, 'de, P: ?Sized + Parser<'de>>: Sized {
    fn parse_next<'p2>(&'p2 mut self) -> anyhow::Result<Option<P::EntryParser<'p2>>>;
    fn exact_size(&self) -> Option<usize> {
        None
    }
    fn map_into_iter<
        C,
        K,
        KF: for<'p2> FnMut(&mut C, P::AnyParser<'p2>) -> anyhow::Result<K>,
        V,
        VF: for<'p2> FnMut(&mut C, K, P::AnyParser<'p2>) -> anyhow::Result<V>,
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
}

pub struct MapIter<'p, 'de, M: MapParser<'p, 'de, P>, P: Parser<'de> + 'p, C, KF, VF> {
    map: M,
    ctx: C,
    key: KF,
    value: VF,
    phantom: PhantomData<(&'p P, &'de ())>,
}

impl<
        'p,
        'de,
        M: MapParser<'p, 'de, P>,
        P: Parser<'de>,
        C,
        K,
        V,
        KF: for<'p2> FnMut(&mut C, P::AnyParser<'p2>) -> anyhow::Result<K>,
        VF: for<'p2> FnMut(&mut C, K, P::AnyParser<'p2>) -> anyhow::Result<V>,
    > Iterator for MapIter<'p, 'de, M, P, C, KF, VF>
{
    type Item = anyhow::Result<V>;

    fn next(&mut self) -> Option<Self::Item> {
        let result: anyhow::Result<Option<V>> = try {
            match self.map.parse_next()? {
                None => None,
                Some(mut p) => {
                    let key = (self.key)(&mut self.ctx, p.parse_key()?)?;
                    let value = (self.value)(&mut self.ctx, key, p.parse_value()?)?;
                    p.parse_end()?;
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

pub trait EntryParser<'p, 'de, P: ?Sized + Parser<'de>> {
    fn parse_key<'p2>(&'p2 mut self) -> anyhow::Result<P::AnyParser<'p2>>;
    fn parse_value<'p2>(&'p2 mut self) -> anyhow::Result<P::AnyParser<'p2>>;
    fn parse_end(self) -> anyhow::Result<()>;
}

pub trait EnumParser<'p, 'de, P: ?Sized + Parser<'de>> {
    fn parse_discriminant<'p2>(&'p2 mut self) -> anyhow::Result<P::AnyParser<'p2>>;
    fn parse_variant<'p2>(
        &'p2 mut self,
        hint: ParseVariantHint,
    ) -> anyhow::Result<ParserView<'p2, 'de, P>>;
    fn parse_end(self) -> anyhow::Result<()>;
}

pub trait SomeParser<'p, 'de, P: ?Sized + Parser<'de>> {
    fn parse_some<'p2>(&'p2 mut self) -> anyhow::Result<<P as Parser<'de>>::AnyParser<'p2>>;
    fn parse_end(self) -> anyhow::Result<()>;
}
pub trait Parser<'de>: Sized {
    type AnyParser<'p>: AnyParser<'p, 'de, Self>
    where
        Self: 'p;
    type SeqParser<'p>: SeqParser<'p, 'de, Self>
    where
        Self: 'p;
    type MapParser<'p>: MapParser<'p, 'de, Self>
    where
        Self: 'p;
    type EntryParser<'p>: EntryParser<'p, 'de, Self>
    where
        Self: 'p;
    type EnumParser<'p>: EnumParser<'p, 'de, Self>
    where
        Self: 'p;
    type SomeParser<'p>: SomeParser<'p, 'de, Self>
    where
        Self: 'p;
}

impl<'p, 'de, P: Parser<'de>> Debug for ParserView<'p, 'de, P> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            ParserView::Primitive(x) => f.debug_tuple("Primitive").field(x).finish(),
            ParserView::String(x) => f.debug_tuple("String").field(x).finish(),
            ParserView::Bytes(x) => f.debug_tuple("Bytes").field(x).finish(),
            ParserView::None => f.debug_tuple("None").finish(),
            ParserView::Some(_) => f.debug_struct("Some").finish_non_exhaustive(),
            ParserView::Seq(_) => f.debug_struct("Seq").finish_non_exhaustive(),
            ParserView::Map(_) => f.debug_struct("Map").finish_non_exhaustive(),
            ParserView::Enum(_) => f.debug_struct("Enum").finish_non_exhaustive(),
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
        write!(f, "Actual type did not match expected type")
    }
}
impl Error for TypeMismatch {}


impl<'p, 'de, P: Parser<'de>> ParserView<'p, 'de, P> {
    pub fn try_into_seq(self) -> anyhow::Result<P::SeqParser<'p>> {
        match self {
            ParserView::Seq(x) => Ok(x),
            unexpected => unexpected.mismatch("seq")?,
        }
    }
    pub fn try_into_map(self) -> anyhow::Result<P::MapParser<'p>> {
        match self {
            ParserView::Map(x) => Ok(x),
            unexpected => unexpected.mismatch("map")?,
        }
    }
    pub fn try_into_string(self) -> anyhow::Result<Cow<'de, str>> {
        match self {
            ParserView::String(x) => Ok(x),
            unexpected => unexpected.mismatch("string")?,
        }
    }
    pub fn mismatch(&self, expected: &'static str) -> anyhow::Result<!> {
        Err(TypeMismatch {
            found: match self {
                ParserView::Primitive(p) => match p {
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
                },
                ParserView::String(_) => "string",
                ParserView::Bytes(_) => "bytes",
                ParserView::None => "none",
                ParserView::Some(_) => "some",
                ParserView::Seq(_) => "seq",
                ParserView::Map(_) => "map",
                ParserView::Enum(_) => "enum",
            },
            expected,
        }
        .into())
    }
}
