use std::error::Error;
use std::fmt::{Debug, Display, Formatter};
use std::marker::PhantomData;

use crate::decode::{
    AnyDecoder, DecodeHint, DecodeVariantHint, Decoder, DecoderView, EntryDecoder, EnumDecoder,
    MapDecoder, SeqDecoder, SomeDecoder,
};

pub struct PoisonDecoder<T>(PhantomData<T>);

pub struct PoisonState {
    poisoned: Result<(), PoisonError>,
}

impl PoisonState {
    pub fn new() -> PoisonState {
        PoisonState { poisoned: Ok(()) }
    }
}

#[derive(Copy, Clone, Debug)]
pub enum PoisonError {
    AnyDecoder,
    SeqDecoder,
    MapDecoder,
    EntryDecoder,
    EntryDecoderRereadKey,
    EntryDecoderRereadValue,
    EntryDecoderForgotKey,
    EntryDecoderForgotValue,
    EnumDecoder,
    EnumDecoderRereadDiscriminant,
    EnumDecoderForgotDiscriminant,
    EnumDecoderRereadVariant,
    EnumDecoderForgotVariant,
    SomeDecoder,
    SomeDecoderRereadSome,
    SomeDecoderForgotSome,
}

impl Display for PoisonError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        Debug::fmt(self, f)
    }
}

impl Error for PoisonError {}

pub struct PoisonGuard<'p> {
    state: Option<&'p mut PoisonState>,
    message: PoisonError,
}

impl PoisonState {
    pub fn check(self) -> Result<(), PoisonError> {
        self.poisoned
    }
}

impl<'p> PoisonGuard<'p> {
    pub fn new(state: &'p mut PoisonState, message: PoisonError) -> Self {
        PoisonGuard {
            state: Some(state),
            message,
        }
    }
    pub fn defuse(&mut self) -> &'p mut PoisonState {
        self.state.take().unwrap()
    }
    pub fn defuse_into(mut self) -> &'p mut PoisonState {
        self.state.take().unwrap()
    }
    pub fn check(&self) -> Result<(), PoisonError> {
        self.state.as_ref().unwrap().poisoned
    }
    pub fn state<'p2>(&'p2 mut self) -> &'p2 mut PoisonState {
        self.state.as_mut().unwrap()
    }
}

impl<'p> Drop for PoisonGuard<'p> {
    fn drop(&mut self) {
        if let Some(state) = self.state.take() {
            state.poisoned = Err(self.message);
        }
    }
}

pub struct PoisonAnyDecoder<'p, 'de, T: 'p + Decoder<'de>> {
    guard: PoisonGuard<'p>,
    inner: T::AnyDecoder<'p>,
}

pub struct PoisonSeqDecoder<'p, 'de, T: 'p + Decoder<'de>> {
    guard: PoisonGuard<'p>,
    inner: T::SeqDecoder<'p>,
}

pub struct PoisonMapDecoder<'p, 'de, T: 'p + Decoder<'de>> {
    guard: PoisonGuard<'p>,
    inner: T::MapDecoder<'p>,
}

pub struct PoisonEntryDecoder<'p, 'de, T: 'p + Decoder<'de>> {
    guard: PoisonGuard<'p>,
    inner: T::EntryDecoder<'p>,
    read_key: bool,
    read_value: bool,
}

pub struct PoisonEnumDecoder<'p, 'de, T: 'p + Decoder<'de>> {
    guard: PoisonGuard<'p>,
    inner: T::EnumDecoder<'p>,
    read_discriminant: bool,
    read_variant: bool,
}

pub struct PoisonSomeDecoder<'p, 'de, T: 'p + Decoder<'de>> {
    guard: PoisonGuard<'p>,
    inner: T::SomeDecoder<'p>,
    read_some: bool,
}

impl<'de, T: Decoder<'de>> Decoder<'de> for PoisonDecoder<T> {
    type AnyDecoder<'p> = PoisonAnyDecoder<'p, 'de, T> where Self: 'p;
    type SeqDecoder<'p> = PoisonSeqDecoder<'p, 'de, T> where Self: 'p;
    type MapDecoder<'p> = PoisonMapDecoder<'p, 'de, T> where Self: 'p;
    type EntryDecoder<'p> = PoisonEntryDecoder<'p, 'de, T> where Self: 'p;
    type EnumDecoder<'p> = PoisonEnumDecoder<'p, 'de, T> where Self: 'p;
    type SomeDecoder<'p> = PoisonSomeDecoder<'p, 'de, T> where Self: 'p;
}

impl<'p, 'de, T: Decoder<'de>> PoisonAnyDecoder<'p, 'de, T> {
    pub fn new(state: &'p mut PoisonState, inner: T::AnyDecoder<'p>) -> Self {
        PoisonAnyDecoder {
            guard: PoisonGuard::new(state, PoisonError::AnyDecoder),
            inner,
        }
    }
}

fn annotate_view<'p, 'de, T: Decoder<'de>>(
    state: &'p mut PoisonState,
    view: DecoderView<'p, 'de, T>,
) -> DecoderView<'p, 'de, PoisonDecoder<T>> {
    match view {
        DecoderView::Primitive(x) => DecoderView::Primitive(x),
        DecoderView::String(x) => DecoderView::String(x),
        DecoderView::Bytes(x) => DecoderView::Bytes(x),
        DecoderView::None => DecoderView::None,
        DecoderView::Some(x) => DecoderView::Some(PoisonSomeDecoder::new(state, x)),
        DecoderView::Seq(x) => DecoderView::Seq(PoisonSeqDecoder::new(state, x)),
        DecoderView::Map(x) => DecoderView::Map(PoisonMapDecoder::new(state, x)),
        DecoderView::Enum(x) => DecoderView::Enum(PoisonEnumDecoder::new(state, x)),
    }
}

impl<'p, 'de, T: Decoder<'de>> AnyDecoder<'p, 'de, PoisonDecoder<T>>
for PoisonAnyDecoder<'p, 'de, T>
{
    fn decode(
        mut self,
        hint: DecodeHint,
    ) -> anyhow::Result<DecoderView<'p, 'de, PoisonDecoder<T>>> {
        self.guard.check()?;
        let inner = self.inner.decode(hint)?;
        let state = self.guard.defuse();
        Ok(annotate_view(state, inner))
    }
}

impl<'p, 'de, T: Decoder<'de>> PoisonSeqDecoder<'p, 'de, T> {
    pub fn new(state: &'p mut PoisonState, inner: T::SeqDecoder<'p>) -> Self {
        PoisonSeqDecoder {
            guard: PoisonGuard::new(state, PoisonError::SeqDecoder),
            inner,
        }
    }
}

impl<'p, 'de, T: Decoder<'de>> SeqDecoder<'p, 'de, PoisonDecoder<T>>
for PoisonSeqDecoder<'p, 'de, T>
{
    fn decode_next<'p2>(&'p2 mut self) -> anyhow::Result<Option<PoisonAnyDecoder<'p2, 'de, T>>> {
        self.guard.check()?;
        if let Some(result) = self.inner.decode_next()? {
            let state = self.guard.state();
            Ok(Some(PoisonAnyDecoder::new(state, result)))
        } else {
            self.guard.defuse();
            Ok(None)
        }
    }
}

impl<'p, 'de, T: Decoder<'de>> PoisonMapDecoder<'p, 'de, T> {
    pub fn new(state: &'p mut PoisonState, inner: T::MapDecoder<'p>) -> Self {
        PoisonMapDecoder {
            guard: PoisonGuard::new(state, PoisonError::MapDecoder),
            inner,
        }
    }
}

impl<'p, 'de, T: Decoder<'de>> MapDecoder<'p, 'de, PoisonDecoder<T>>
for PoisonMapDecoder<'p, 'de, T>
{
    fn decode_next<'p2>(&'p2 mut self) -> anyhow::Result<Option<PoisonEntryDecoder<'p2, 'de, T>>> {
        self.guard.check()?;
        if let Some(result) = self.inner.decode_next()? {
            let state = self.guard.state();
            Ok(Some(PoisonEntryDecoder::new(state, result)))
        } else {
            self.guard.defuse();
            Ok(None)
        }
    }
}

impl<'p, 'de, T: Decoder<'de>> PoisonEntryDecoder<'p, 'de, T> {
    pub fn new(state: &'p mut PoisonState, inner: T::EntryDecoder<'p>) -> Self {
        PoisonEntryDecoder {
            guard: PoisonGuard::new(state, PoisonError::EntryDecoder),
            inner,
            read_key: false,
            read_value: false,
        }
    }
}

impl<'p, 'de, T: Decoder<'de>> EntryDecoder<'p, 'de, PoisonDecoder<T>>
for PoisonEntryDecoder<'p, 'de, T>
{
    fn decode_key<'p2>(&'p2 mut self) -> anyhow::Result<PoisonAnyDecoder<'p2, 'de, T>> {
        if self.read_key {
            return Err(PoisonError::EntryDecoderRereadKey.into());
        }
        self.read_key = true;
        self.guard.check()?;
        let result = self.inner.decode_key()?;
        let state = self.guard.state();
        Ok(PoisonAnyDecoder::new(state, result))
    }

    fn decode_value<'p2>(&'p2 mut self) -> anyhow::Result<PoisonAnyDecoder<'p2, 'de, T>> {
        if !self.read_key {
            return Err(PoisonError::EntryDecoderForgotKey.into());
        }
        if self.read_value {
            return Err(PoisonError::EntryDecoderRereadValue.into());
        }
        self.read_value = true;
        self.guard.check()?;
        let result = self.inner.decode_value()?;
        let state = self.guard.state();
        Ok(PoisonAnyDecoder::new(state, result))
    }

    fn decode_end(mut self) -> anyhow::Result<()> {
        if !self.read_value {
            return Err(PoisonError::EntryDecoderForgotValue.into());
        }
        self.guard.check()?;
        self.guard.defuse();
        Ok(())
    }
}

impl<'p, 'de, T: Decoder<'de>> PoisonEnumDecoder<'p, 'de, T> {
    pub fn new(state: &'p mut PoisonState, inner: T::EnumDecoder<'p>) -> Self {
        PoisonEnumDecoder {
            guard: PoisonGuard::new(state, PoisonError::EnumDecoder),
            inner,
            read_discriminant: false,
            read_variant: false,
        }
    }
}

impl<'p, 'de, T: Decoder<'de>> EnumDecoder<'p, 'de, PoisonDecoder<T>>
for PoisonEnumDecoder<'p, 'de, T>
{
    fn decode_discriminant<'p2>(&'p2 mut self) -> anyhow::Result<PoisonAnyDecoder<'p2, 'de, T>> {
        if self.read_discriminant {
            return Err(PoisonError::EnumDecoderRereadDiscriminant.into());
        }
        self.read_discriminant = true;
        self.guard.check()?;
        let result = self.inner.decode_discriminant()?;
        let state = self.guard.state();
        Ok(PoisonAnyDecoder::new(state, result))
    }

    fn decode_variant<'p2>(
        &'p2 mut self,
        hint: DecodeVariantHint,
    ) -> anyhow::Result<DecoderView<'p2, 'de, PoisonDecoder<T>>> {
        if !self.read_discriminant {
            return Err(PoisonError::EnumDecoderForgotDiscriminant.into());
        }
        if self.read_variant {
            return Err(PoisonError::EnumDecoderRereadVariant.into());
        }
        self.read_variant = true;
        self.guard.check()?;
        let result = self.inner.decode_variant(hint)?;
        let state = self.guard.state();
        Ok(annotate_view(state, result))
    }

    fn decode_end(mut self) -> anyhow::Result<()> {
        if !self.read_variant {
            return Err(PoisonError::EnumDecoderForgotVariant.into());
        }
        self.guard.check()?;
        self.inner.decode_end()?;
        self.guard.defuse();
        Ok(())
    }
}

impl<'p, 'de, T: Decoder<'de>> PoisonSomeDecoder<'p, 'de, T> {
    pub fn new(state: &'p mut PoisonState, inner: T::SomeDecoder<'p>) -> Self {
        PoisonSomeDecoder {
            guard: PoisonGuard::new(state, PoisonError::SomeDecoder),
            inner,
            read_some: false,
        }
    }
}

impl<'p, 'de, T: Decoder<'de>> SomeDecoder<'p, 'de, PoisonDecoder<T>>
for PoisonSomeDecoder<'p, 'de, T>
{
    fn decode_some<'p2>(&'p2 mut self) -> anyhow::Result<PoisonAnyDecoder<'p2, 'de, T>> {
        if self.read_some {
            return Err(PoisonError::SomeDecoderRereadSome.into());
        }
        self.read_some = true;
        self.guard.check()?;
        let inner = self.inner.decode_some()?;
        Ok(PoisonAnyDecoder::new(self.guard.state(), inner))
    }

    fn decode_end(mut self) -> anyhow::Result<()> {
        if !self.read_some {
            return Err(PoisonError::SomeDecoderForgotSome.into());
        }
        self.guard.check()?;
        self.inner.decode_end()?;
        self.guard.defuse();
        Ok(())
    }
}
