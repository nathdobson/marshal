use std::error::Error;
use std::fmt::{Display, Formatter};

use crate::decode::{DecodeHint, DecodeVariantHint, SimpleDecoderView, SpecDecoder};

pub struct DepthBudgetDecoder<D> {
    inner: D,
}

pub struct WithDepthBudget<T> {
    budget: usize,
    inner: T,
}

#[derive(Debug)]
pub struct OverflowError;

impl Display for OverflowError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "parsing depth limit exceeded")
    }
}

impl Error for OverflowError {}

impl<T> WithDepthBudget<T> {
    #[inline]
    pub fn new(budget: usize, inner: T) -> Self {
        WithDepthBudget { budget, inner }
    }
}

impl<'de, D: SpecDecoder<'de>> DepthBudgetDecoder<D> {
    #[inline]
    pub fn new(inner: D) -> Self {
        DepthBudgetDecoder { inner }
    }
    #[inline]
    fn wrap_view<'p>(
        budget: usize,
        view: SimpleDecoderView<'de, D>,
    ) -> SimpleDecoderView<'de, Self> {
        match view {
            SimpleDecoderView::Primitive(x) => SimpleDecoderView::Primitive(x),
            SimpleDecoderView::String(x) => SimpleDecoderView::String(x),
            SimpleDecoderView::Bytes(x) => SimpleDecoderView::Bytes(x),
            SimpleDecoderView::None => SimpleDecoderView::None,
            SimpleDecoderView::Some(x) => SimpleDecoderView::Some(WithDepthBudget::new(budget, x)),
            SimpleDecoderView::Seq(x) => SimpleDecoderView::Seq(WithDepthBudget::new(budget, x)),
            SimpleDecoderView::Map(x) => SimpleDecoderView::Map(WithDepthBudget::new(budget, x)),
            SimpleDecoderView::Enum(x) => SimpleDecoderView::Enum(WithDepthBudget::new(budget, x)),
        }
    }
    #[inline]
    pub fn inner(&self) -> &D {
        &self.inner
    }
    #[inline]
    pub fn inner_mut(&mut self) -> &mut  D {
        &mut self.inner
    }
    #[inline]
    pub fn end(self) -> anyhow::Result<D> {
        Ok(self.inner)
    }
}

impl<'de, D: SpecDecoder<'de>> SpecDecoder<'de> for DepthBudgetDecoder<D> {
    type AnyDecoder = WithDepthBudget<D::AnyDecoder>;
    type SeqDecoder = WithDepthBudget<D::SeqDecoder>;
    type MapDecoder = WithDepthBudget<D::MapDecoder>;
    type KeyDecoder = WithDepthBudget<D::KeyDecoder>;
    type ValueDecoder = WithDepthBudget<D::ValueDecoder>;
    type DiscriminantDecoder = WithDepthBudget<D::DiscriminantDecoder>;
    type VariantDecoder = WithDepthBudget<D::VariantDecoder>;
    type EnumCloser = WithDepthBudget<D::EnumCloser>;
    type SomeDecoder = WithDepthBudget<D::SomeDecoder>;
    type SomeCloser = WithDepthBudget<D::SomeCloser>;

    #[inline]
    fn decode(
        &mut self,
        any: Self::AnyDecoder,
        hint: DecodeHint,
    ) -> anyhow::Result<SimpleDecoderView<'de, Self>> {
        Ok(Self::wrap_view(
            any.budget.checked_sub(1).ok_or(OverflowError)?,
            self.inner.decode(any.inner, hint)?,
        ))
    }

    #[inline]
    fn is_human_readable(&self) -> bool {
        self.inner.is_human_readable()
    }

    #[inline]
    fn decode_seq_next(
        &mut self,
        seq: &mut Self::SeqDecoder,
    ) -> anyhow::Result<Option<Self::AnyDecoder>> {
        if let Some(next) = self.inner.decode_seq_next(&mut seq.inner)? {
            Ok(Some(WithDepthBudget::new(seq.budget, next)))
        } else {
            Ok(None)
        }
    }

    #[inline]
    fn decode_seq_end(&mut self, seq: Self::SeqDecoder) -> anyhow::Result<()> {
        self.inner.decode_seq_end(seq.inner)
    }

    #[inline]
    fn decode_map_next(
        &mut self,
        map: &mut Self::MapDecoder,
    ) -> anyhow::Result<Option<Self::KeyDecoder>> {
        if let Some(next) = self.inner.decode_map_next(&mut map.inner)? {
            Ok(Some(WithDepthBudget::new(map.budget, next)))
        } else {
            Ok(None)
        }
    }

    #[inline]
    fn decode_map_end(&mut self, map: Self::MapDecoder) -> anyhow::Result<()> {
        self.inner.decode_map_end(map.inner)
    }

    #[inline]
    fn decode_entry_key(
        &mut self,
        key: Self::KeyDecoder,
    ) -> anyhow::Result<(Self::AnyDecoder, Self::ValueDecoder)> {
        let (any, value) = self.inner.decode_entry_key(key.inner)?;
        Ok((
            WithDepthBudget::new(key.budget, any),
            WithDepthBudget::new(key.budget, value),
        ))
    }

    #[inline]
    fn decode_entry_value(
        &mut self,
        value: Self::ValueDecoder,
    ) -> anyhow::Result<Self::AnyDecoder> {
        Ok(WithDepthBudget::new(
            value.budget,
            self.inner.decode_entry_value(value.inner)?,
        ))
    }

    #[inline]
    fn decode_enum_discriminant(
        &mut self,
        disc: Self::DiscriminantDecoder,
    ) -> anyhow::Result<(Self::AnyDecoder, Self::VariantDecoder)> {
        let (any, variant) = self.inner.decode_enum_discriminant(disc.inner)?;
        Ok((
            WithDepthBudget::new(disc.budget, any),
            WithDepthBudget::new(disc.budget, variant),
        ))
    }

    #[inline]
    fn decode_enum_variant(
        &mut self,
        variant: Self::VariantDecoder,
        hint: DecodeVariantHint,
    ) -> anyhow::Result<(SimpleDecoderView<'de, Self>, Self::EnumCloser)> {
        let (any, closer) = self.inner.decode_enum_variant(variant.inner, hint)?;
        Ok((
            Self::wrap_view(variant.budget, any),
            WithDepthBudget::new(variant.budget, closer),
        ))
    }

    #[inline]
    fn decode_enum_end(&mut self, closer: Self::EnumCloser) -> anyhow::Result<()> {
        self.inner.decode_enum_end(closer.inner)
    }

    #[inline]
    fn decode_some_inner(
        &mut self,
        some: Self::SomeDecoder,
    ) -> anyhow::Result<(Self::AnyDecoder, Self::SomeCloser)> {
        let (any, closer) = self.inner.decode_some_inner(some.inner)?;
        Ok((
            WithDepthBudget::new(some.budget, any),
            WithDepthBudget::new(some.budget, closer),
        ))
    }

    #[inline]
    fn decode_some_end(&mut self, closer: Self::SomeCloser) -> anyhow::Result<()> {
        self.inner.decode_some_end(closer.inner)
    }
}
