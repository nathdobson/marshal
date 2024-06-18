use crate::decode::{ DecodeHint, DecodeVariantHint, Decoder, SimpleDecoderView};
use std::fmt::{Display, Formatter};

pub struct PoisonDecoder<D> {
    inner: D,
    depth: usize,
}

pub struct PoisonWrapper<T> {
    inner: T,
    depth: usize,
}

#[derive(Debug)]
pub enum PoisonError {
    UnexpectedDecodeState,
}
impl Display for PoisonError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        std::fmt::Debug::fmt(self, f)
    }
}
impl std::error::Error for PoisonError {}

impl<'de, D: Decoder<'de>> PoisonDecoder<D> {
    pub fn new(inner: D) -> Self {
        PoisonDecoder { inner, depth: 0 }
    }
    pub fn start<'p>(&'p mut self, inner: D::AnyDecoder) -> <Self as Decoder<'de>>::AnyDecoder {
        self.push(inner)
    }
    pub fn end(self) -> anyhow::Result<D> {
        if self.depth == 0 {
            Ok(self.inner)
        } else {
            Err(PoisonError::UnexpectedDecodeState.into())
        }
    }
    fn push<T>(&mut self, inner: T) -> PoisonWrapper<T> {
        self.depth += 1;
        PoisonWrapper {
            depth: self.depth,
            inner,
        }
    }
    fn pop<T>(&mut self, wrapper: PoisonWrapper<T>) -> anyhow::Result<T> {
        if wrapper.depth == self.depth {
            self.depth -= 1;
            Ok(wrapper.inner)
        } else {
            Err(PoisonError::UnexpectedDecodeState.into())
        }
    }
    fn peek<'a, T>(&self, wrapper: &'a mut PoisonWrapper<T>) -> anyhow::Result<&'a mut T> {
        if wrapper.depth == self.depth {
            Ok(&mut wrapper.inner)
        } else {
            Err(PoisonError::UnexpectedDecodeState.into())
        }
    }
    fn wrap_view(&mut self, view: SimpleDecoderView<'de, D>) -> SimpleDecoderView<'de, Self> {
        match view {
            SimpleDecoderView::Primitive(x) => SimpleDecoderView::Primitive(x),
            SimpleDecoderView::String(x) => SimpleDecoderView::String(x),
            SimpleDecoderView::Bytes(x) => SimpleDecoderView::Bytes(x),
            SimpleDecoderView::None => SimpleDecoderView::None,
            SimpleDecoderView::Some(x) => SimpleDecoderView::Some(self.push(x)),
            SimpleDecoderView::Seq(x) => SimpleDecoderView::Seq(self.push(x)),
            SimpleDecoderView::Map(x) => SimpleDecoderView::Map(self.push(x)),
            SimpleDecoderView::Enum(x) => SimpleDecoderView::Enum(self.push(x)),
        }
    }
}

impl<'de, D: Decoder<'de>> Decoder<'de> for PoisonDecoder<D> {
    type AnyDecoder = PoisonWrapper<D::AnyDecoder>;
    type SeqDecoder = PoisonWrapper<D::SeqDecoder>;
    type MapDecoder = PoisonWrapper<D::MapDecoder>;
    type KeyDecoder = PoisonWrapper<D::KeyDecoder>;
    type ValueDecoder = PoisonWrapper<D::ValueDecoder>;
    type DiscriminantDecoder = PoisonWrapper<D::DiscriminantDecoder>;
    type VariantDecoder = PoisonWrapper<D::VariantDecoder>;
    type EnumCloser = PoisonWrapper<D::EnumCloser>;
    type SomeDecoder = PoisonWrapper<D::SomeDecoder>;
    type SomeCloser = PoisonWrapper<D::SomeCloser>;

    fn decode(
        &mut self,
        any: Self::AnyDecoder,
        hint: DecodeHint,
    ) -> anyhow::Result<SimpleDecoderView<'de, Self>> {
        let any = self.pop(any)?;
        let decoder = self.inner.decode(any, hint)?;
        Ok(self.wrap_view(decoder))
    }

    fn is_human_readable(&self) -> bool {
        self.inner.is_human_readable()
    }

    fn decode_seq_next(
        &mut self,
        seq: &mut Self::SeqDecoder,
    ) -> anyhow::Result<Option<Self::AnyDecoder>> {
        let seq_inner = self.peek(seq)?;
        if let Some(decoder) = self.inner.decode_seq_next(seq_inner)? {
            Ok(Some(self.push(decoder)))
        } else {
            Ok(None)
        }
    }

    fn decode_seq_end(&mut self, seq: Self::SeqDecoder) -> anyhow::Result<()> {
        let seq = self.pop(seq)?;
        self.inner.decode_seq_end(seq)
    }

    fn decode_map_next(
        &mut self,
        map: &mut Self::MapDecoder,
    ) -> anyhow::Result<Option<Self::KeyDecoder>> {
        let map = self.peek(map)?;
        let decoder = self.inner.decode_map_next(map)?;
        Ok(decoder.map(|decoder| self.push(decoder)))
    }

    fn decode_map_end(&mut self, map: Self::MapDecoder) -> anyhow::Result<()> {
        let map = self.pop(map)?;
        self.inner.decode_map_end(map)
    }

    fn decode_entry_key(
        &mut self,
        key: Self::KeyDecoder,
    ) -> anyhow::Result<(Self::AnyDecoder, Self::ValueDecoder)> {
        let key = self.pop(key)?;
        let (decoder, value) = self.inner.decode_entry_key(key)?;
        let value = self.push(value);
        let decoder = self.push(decoder);
        Ok((decoder, value))
    }

    fn decode_entry_value(
        &mut self,
        value: Self::ValueDecoder,
    ) -> anyhow::Result<Self::AnyDecoder> {
        let value = self.pop(value)?;
        let decoder = self.inner.decode_entry_value(value)?;
        Ok(self.push(decoder))
    }

    fn decode_enum_discriminant(
        &mut self,
        disc: Self::DiscriminantDecoder,
    ) -> anyhow::Result<(Self::AnyDecoder, Self::VariantDecoder)> {
        let disc = self.pop(disc)?;
        let (decoder, variant) = self.inner.decode_enum_discriminant(disc)?;
        let variant = self.push(variant);
        let decoder = self.push(decoder);
        Ok((decoder, variant))
    }

    fn decode_enum_variant(
        &mut self,
        variant: Self::VariantDecoder,
        hint: DecodeVariantHint,
    ) -> anyhow::Result<(SimpleDecoderView<'de, Self>, Self::EnumCloser)> {
        let variant = self.pop(variant)?;
        let (decoder, closer) = self.inner.decode_enum_variant(variant, hint)?;
        let closer = self.push(closer);
        let decoder = self.wrap_view(decoder);
        Ok((decoder, closer))
    }

    fn decode_enum_end(&mut self, closer: Self::EnumCloser) -> anyhow::Result<()> {
        let closer = self.pop(closer)?;
        self.inner.decode_enum_end(closer)?;
        Ok(())
    }

    fn decode_some_inner(
        &mut self,
        some: Self::SomeDecoder,
    ) -> anyhow::Result<(Self::AnyDecoder, Self::SomeCloser)> {
        let some = self.pop(some)?;
        let (decoder, closer) = self.inner.decode_some_inner(some)?;
        let closer = self.push(closer);
        let decoder = self.push(decoder);
        Ok((decoder, closer))
    }

    fn decode_some_end(&mut self, closer: Self::SomeCloser) -> anyhow::Result<()> {
        let closer = self.pop(closer)?;
        self.inner.decode_some_end(closer)?;
        Ok(())
    }
}
