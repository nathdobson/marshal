use std::error::Error;
use std::fmt::{Display, Formatter};
use std::marker::PhantomData;

use crate::decode::{
    AnyDecoder, EntryDecoder, EnumDecoder, MapDecoder, DecodeHint, Decoder, DecoderView, DecodeVariantHint,
    SeqDecoder, SomeDecoder,
};

pub struct DepthBudgetDecoder<T>(PhantomData<T>);

pub struct WithDepthBudget<T> {
    depth_budget: usize,
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

impl<'de, T: Decoder<'de>> Decoder<'de> for DepthBudgetDecoder<T> {
    type AnyDecoder<'p> = WithDepthBudget<T::AnyDecoder<'p>> where Self: 'p;
    type SeqDecoder<'p> = WithDepthBudget<T::SeqDecoder<'p>> where Self: 'p;
    type MapDecoder<'p> = WithDepthBudget<T::MapDecoder<'p>> where Self: 'p;
    type EntryDecoder<'p> = WithDepthBudget<T::EntryDecoder<'p>> where Self: 'p;
    type EnumDecoder<'p> = WithDepthBudget<T::EnumDecoder<'p>> where Self: 'p;
    type SomeDecoder<'p> = WithDepthBudget<T::SomeDecoder<'p>> where Self: 'p;
}

fn annotate<'p, 'de, T: Decoder<'de>>(
    depth_budget: usize,
    view: DecoderView<'p, 'de, T>,
) -> anyhow::Result<DecoderView<'p, 'de, DepthBudgetDecoder<T>>> {
    let depth_budget: Result<usize, OverflowError> =
        depth_budget.checked_sub(1).ok_or(OverflowError);
    Ok(match view {
        DecoderView::Primitive(x) => DecoderView::Primitive(x),
        DecoderView::String(x) => DecoderView::String(x),
        DecoderView::Bytes(x) => DecoderView::Bytes(x),
        DecoderView::None => DecoderView::None,
        DecoderView::Some(inner) => DecoderView::Some(WithDepthBudget {
            depth_budget: depth_budget?,
            inner,
        }),
        DecoderView::Seq(inner) => DecoderView::Seq(WithDepthBudget {
            depth_budget: depth_budget?,
            inner,
        }),
        DecoderView::Map(inner) => DecoderView::Map(WithDepthBudget {
            depth_budget: depth_budget?,
            inner,
        }),
        DecoderView::Enum(inner) => DecoderView::Enum(WithDepthBudget {
            depth_budget: depth_budget?,
            inner,
        }),
    })
}

impl<T> WithDepthBudget<T> {
    pub fn new(depth_budget: usize, inner: T) -> Self {
        WithDepthBudget {
            depth_budget,
            inner,
        }
    }
}

impl<'p, 'de, T: Decoder<'de>> AnyDecoder<'p, 'de, DepthBudgetDecoder<T>>
    for WithDepthBudget<T::AnyDecoder<'p>>
{
    fn decode(self, hint: DecodeHint) -> anyhow::Result<DecoderView<'p, 'de, DepthBudgetDecoder<T>>> {
        annotate(self.depth_budget, self.inner.decode(hint)?)
    }
}

impl<'p, 'de, T: Decoder<'de>> SeqDecoder<'p, 'de, DepthBudgetDecoder<T>>
    for WithDepthBudget<T::SeqDecoder<'p>>
{
    fn decode_next<'p2>(
        &'p2 mut self,
    ) -> anyhow::Result<Option<WithDepthBudget<T::AnyDecoder<'p2>>>> {
        if let Some(inner) = self.inner.decode_next()? {
            Ok(Some(WithDepthBudget {
                depth_budget: self.depth_budget,
                inner,
            }))
        } else {
            Ok(None)
        }
    }
}

impl<'p, 'de, T: Decoder<'de>> MapDecoder<'p, 'de, DepthBudgetDecoder<T>>
    for WithDepthBudget<T::MapDecoder<'p>>
{
    fn decode_next<'p2>(
        &'p2 mut self,
    ) -> anyhow::Result<Option<WithDepthBudget<T::EntryDecoder<'p2>>>> {
        if let Some(inner) = self.inner.decode_next()? {
            Ok(Some(WithDepthBudget {
                depth_budget: self.depth_budget,
                inner,
            }))
        } else {
            Ok(None)
        }
    }
}

impl<'p, 'de, T: Decoder<'de>> EntryDecoder<'p, 'de, DepthBudgetDecoder<T>>
    for WithDepthBudget<T::EntryDecoder<'p>>
{
    fn decode_key<'p2>(&'p2 mut self) -> anyhow::Result<WithDepthBudget<T::AnyDecoder<'p2>>> {
        Ok(WithDepthBudget {
            depth_budget: self.depth_budget,
            inner: self.inner.decode_key()?,
        })
    }

    fn decode_value<'p2>(&'p2 mut self) -> anyhow::Result<WithDepthBudget<T::AnyDecoder<'p2>>> {
        Ok(WithDepthBudget {
            depth_budget: self.depth_budget,
            inner: self.inner.decode_value()?,
        })
    }

    fn decode_end(self) -> anyhow::Result<()> {
        Ok(self.inner.decode_end()?)
    }
}

impl<'p, 'de, T: Decoder<'de>> EnumDecoder<'p, 'de, DepthBudgetDecoder<T>>
    for WithDepthBudget<T::EnumDecoder<'p>>
{
    fn decode_discriminant<'p2>(
        &'p2 mut self,
    ) -> anyhow::Result<WithDepthBudget<T::AnyDecoder<'p2>>> {
        Ok(WithDepthBudget {
            depth_budget: self.depth_budget,
            inner: self.inner.decode_discriminant()?,
        })
    }

    fn decode_variant<'p2>(
        &'p2 mut self,
        hint: DecodeVariantHint,
    ) -> anyhow::Result<DecoderView<'p2, 'de, DepthBudgetDecoder<T>>> {
        Ok(annotate(
            self.depth_budget,
            self.inner.decode_variant(hint)?,
        )?)
    }

    fn decode_end(self) -> anyhow::Result<()> {
        Ok(self.inner.decode_end()?)
    }
}
impl<'p, 'de, T: Decoder<'de>> SomeDecoder<'p, 'de, DepthBudgetDecoder<T>>
    for WithDepthBudget<<T as Decoder<'de>>::SomeDecoder<'p>>
{
    fn decode_some<'p2>(
        &'p2 mut self,
    ) -> anyhow::Result<<DepthBudgetDecoder<T> as Decoder<'de>>::AnyDecoder<'p2>> {
        Ok(WithDepthBudget {
            depth_budget: self.depth_budget,
            inner: self.inner.decode_some()?,
        })
    }

    fn decode_end(self) -> anyhow::Result<()> {
        Ok(self.inner.decode_end()?)
    }
}
