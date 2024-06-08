use std::borrow::Cow;
use std::marker::PhantomData;

use crate::decode::{
    AnyDecoder, EntryDecoder, EnumDecoder, MapDecoder, DecodeHint, DecodeVariantHint, Decoder, DecoderView,
    SeqDecoder, SomeDecoder,
};
use crate::Primitive;

pub struct SimpleDecoderAdapter<T> {
    inner: PhantomData<T>,
}

pub enum SimpleDecoderView<'de, P: ?Sized + SimpleDecoder<'de>> {
    Primitive(Primitive),
    String(Cow<'de, str>),
    Bytes(Cow<'de, [u8]>),
    None,
    Some(P::SomeDecoder),
    Seq(P::SeqDecoder),
    Map(P::MapDecoder),
    Enum(P::DiscriminantDecoder),
}

pub trait SimpleDecoder<'de> {
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

    fn decode_map_next(
        &mut self,
        map: &mut Self::MapDecoder,
    ) -> anyhow::Result<Option<Self::KeyDecoder>>;

    fn decode_entry_key(
        &mut self,
        key: Self::KeyDecoder,
    ) -> anyhow::Result<(Self::AnyDecoder, Self::ValueDecoder)>;

    fn decode_entry_value(&mut self, value: Self::ValueDecoder) -> anyhow::Result<Self::AnyDecoder>;

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

pub struct SimpleAnyDecoder<'p, 'de, T: SimpleDecoder<'de>> {
    this: &'p mut T,
    any: T::AnyDecoder,
}

pub struct SimpleSeqDecoder<'p, 'de, T: SimpleDecoder<'de>> {
    this: &'p mut T,
    seq: T::SeqDecoder,
}

pub struct SimpleMapDecoder<'p, 'de, T: SimpleDecoder<'de>> {
    this: &'p mut T,
    map: T::MapDecoder,
}

pub struct SimpleEntryDecoder<'p, 'de, T: SimpleDecoder<'de>> {
    this: &'p mut T,
    key: Option<T::KeyDecoder>,
    value: Option<T::ValueDecoder>,
}

pub struct SimpleEnumDecoder<'p, 'de, T: SimpleDecoder<'de>> {
    this: &'p mut T,
    discriminant: Option<T::DiscriminantDecoder>,
    variant: Option<T::VariantDecoder>,
    closer: Option<T::EnumCloser>,
}

pub struct SimpleSomeDecoder<'p, 'de, T: SimpleDecoder<'de>> {
    this: &'p mut T,
    some_decoder: Option<T::SomeDecoder>,
    some_closer: Option<T::SomeCloser>,
}

impl<'de, T> Decoder<'de> for SimpleDecoderAdapter<T>
where
    T: SimpleDecoder<'de>,
{
    type AnyDecoder<'p> = SimpleAnyDecoder<'p, 'de,T> where T:'p;
    type SeqDecoder<'p> = SimpleSeqDecoder<'p, 'de, T> where T:'p;
    type MapDecoder<'p> = SimpleMapDecoder<'p, 'de, T> where Self: 'p;
    type EntryDecoder<'p> = SimpleEntryDecoder<'p, 'de, T> where Self: 'p;
    type EnumDecoder<'p> = SimpleEnumDecoder<'p,'de, T> where Self: 'p;
    type SomeDecoder<'p> = SimpleSomeDecoder<'p,'de,T> where Self:'p;
}

impl<'de, T: SimpleDecoder<'de>> SimpleDecoderView<'de, T> {
    fn wrap<'p>(self, this: &'p mut T) -> DecoderView<'p, 'de, SimpleDecoderAdapter<T>> {
        match self {
            SimpleDecoderView::Primitive(x) => DecoderView::Primitive(x),
            SimpleDecoderView::String(x) => DecoderView::String(x),
            SimpleDecoderView::Bytes(x) => DecoderView::Bytes(x),
            SimpleDecoderView::None => DecoderView::None,
            SimpleDecoderView::Some(some) => DecoderView::Some(SimpleSomeDecoder {
                this,
                some_decoder: Some(some),
                some_closer: None,
            }),
            SimpleDecoderView::Seq(seq) => DecoderView::Seq(SimpleSeqDecoder { this, seq }),
            SimpleDecoderView::Map(map) => DecoderView::Map(SimpleMapDecoder { this, map }),
            SimpleDecoderView::Enum(data) => DecoderView::Enum(SimpleEnumDecoder {
                this,
                discriminant: Some(data),
                variant: None,
                closer: None,
            }),
        }
    }
}

impl<'p, 'de, T> AnyDecoder<'p, 'de, SimpleDecoderAdapter<T>> for SimpleAnyDecoder<'p, 'de, T>
where
    T: SimpleDecoder<'de>,
{
    fn decode(self, hint: DecodeHint) -> anyhow::Result<DecoderView<'p, 'de, SimpleDecoderAdapter<T>>> {
        Ok(self.this.decode(self.any, hint)?.wrap(self.this))
    }
}

impl<'p, 'de, T> SeqDecoder<'p, 'de, SimpleDecoderAdapter<T>> for SimpleSeqDecoder<'p, 'de, T>
where
    T: SimpleDecoder<'de>,
{
    fn decode_next<'p2>(&'p2 mut self) -> anyhow::Result<Option<SimpleAnyDecoder<'p2, 'de, T>>> {
        if let Some(any) = self.this.decode_seq_next(&mut self.seq)? {
            Ok(Some(SimpleAnyDecoder {
                this: self.this,
                any,
            }))
        } else {
            Ok(None)
        }
    }
}

impl<'p, 'de, T> MapDecoder<'p, 'de, SimpleDecoderAdapter<T>> for SimpleMapDecoder<'p, 'de, T>
where
    T: SimpleDecoder<'de>,
{
    fn decode_next<'p2>(&'p2 mut self) -> anyhow::Result<Option<SimpleEntryDecoder<'p2, 'de, T>>> {
        if let Some(data) = self.this.decode_map_next(&mut self.map)? {
            Ok(Some(SimpleEntryDecoder {
                this: self.this,
                key: Some(data),
                value: None,
            }))
        } else {
            Ok(None)
        }
    }
}
impl<'p, 'de, T> EntryDecoder<'p, 'de, SimpleDecoderAdapter<T>> for SimpleEntryDecoder<'p, 'de, T>
where
    T: SimpleDecoder<'de>,
{
    fn decode_key<'p2>(&'p2 mut self) -> anyhow::Result<SimpleAnyDecoder<'p2, 'de, T>> {
        let (key, value) = self.this.decode_entry_key(self.key.take().unwrap())?;
        self.value = Some(value);
        Ok(SimpleAnyDecoder {
            this: self.this,
            any: key,
        })
    }

    fn decode_value<'p2>(&'p2 mut self) -> anyhow::Result<SimpleAnyDecoder<'p2, 'de, T>> {
        let value = self.value.take().unwrap();
        let value = self.this.decode_entry_value(value)?;
        Ok(SimpleAnyDecoder {
            this: self.this,
            any: value,
        })
    }

    fn decode_end(mut self) -> anyhow::Result<()> {
        Ok(())
    }
}

impl<'p, 'de, T> EnumDecoder<'p, 'de, SimpleDecoderAdapter<T>> for SimpleEnumDecoder<'p, 'de, T>
where
    T: SimpleDecoder<'de>,
{
    fn decode_discriminant<'p2>(&'p2 mut self) -> anyhow::Result<SimpleAnyDecoder<'p2, 'de, T>> {
        let (discriminant, variant) = self
            .this
            .decode_enum_discriminant(self.discriminant.take().unwrap())?;
        self.variant = Some(variant);
        Ok(SimpleAnyDecoder {
            this: self.this,
            any: discriminant,
        })
    }

    fn decode_variant<'p2>(
        &'p2 mut self,
        hint: DecodeVariantHint,
    ) -> anyhow::Result<DecoderView<'p2, 'de, SimpleDecoderAdapter<T>>> {
        let (data, closer) = self
            .this
            .decode_enum_variant(self.variant.take().unwrap(), hint)?;
        self.closer = Some(closer);
        Ok(data.wrap(self.this))
    }

    fn decode_end(mut self) -> anyhow::Result<()> {
        self.this.decode_enum_end(self.closer.take().unwrap())
    }
}

impl<'p, 'de, T> SomeDecoder<'p, 'de, SimpleDecoderAdapter<T>> for SimpleSomeDecoder<'p, 'de, T>
where
    T: SimpleDecoder<'de>,
{
    fn decode_some<'p2>(&'p2 mut self) -> anyhow::Result<SimpleAnyDecoder<'p2, 'de, T>> {
        let (any, closer) = self
            .this
            .decode_some_inner(self.some_decoder.take().unwrap())?;
        self.some_closer = Some(closer);
        Ok(SimpleAnyDecoder::new(self.this, any))
    }

    fn decode_end(mut self) -> anyhow::Result<()> {
        self.this.decode_some_end(self.some_closer.take().unwrap())
    }
}

impl<'p, 'de, T: SimpleDecoder<'de>> SimpleAnyDecoder<'p, 'de, T> {
    pub fn new(decoder: &'p mut T, any: T::AnyDecoder) -> Self {
        SimpleAnyDecoder { this: decoder, any }
    }
}
