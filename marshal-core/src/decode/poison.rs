use std::error::Error;
use std::fmt::{Debug, Display, Formatter};
use std::marker::PhantomData;

use crate::decode::{
    AnyDecoder, EntryDecoder, EnumDecoder, MapDecoder, DecodeHint, DecodeVariantHint, Decoder, DecoderView,
    SeqDecoder, SomeDecoder,
};

pub struct PoisonParser<T>(PhantomData<T>);

pub struct PoisonState {
    poisoned: Result<(), PoisonError>,
}

impl PoisonState {
    pub fn new() -> PoisonState {
        PoisonState { poisoned: Ok(()) }
    }
}

#[derive(Copy, Clone, Debug)]
pub struct PoisonError(&'static str);

impl Display for PoisonError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        Debug::fmt(self, f)
    }
}

impl Error for PoisonError {}

pub struct PoisonGuard<'p> {
    state: Option<&'p mut PoisonState>,
    message: &'static str,
}

impl PoisonState {
    pub fn check(self) -> Result<(), PoisonError> {
        self.poisoned
    }
}

impl<'p> PoisonGuard<'p> {
    pub fn new(state: &'p mut PoisonState, message: &'static str) -> Self {
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
            state.poisoned = Err(PoisonError(self.message));
        }
    }
}

pub struct PoisonAnyParser<'p, 'de, T: 'p + Decoder<'de>> {
    guard: PoisonGuard<'p>,
    inner: T::AnyDecoder<'p>,
}

pub struct PoisonSeqParser<'p, 'de, T: 'p + Decoder<'de>> {
    guard: PoisonGuard<'p>,
    inner: T::SeqDecoder<'p>,
}

pub struct PoisonMapParser<'p, 'de, T: 'p + Decoder<'de>> {
    guard: PoisonGuard<'p>,
    inner: T::MapDecoder<'p>,
}

pub struct PoisonEntryParser<'p, 'de, T: 'p + Decoder<'de>> {
    guard: PoisonGuard<'p>,
    inner: T::EntryDecoder<'p>,
    read_key: bool,
    read_value: bool,
}

pub struct PoisonEnumParser<'p, 'de, T: 'p + Decoder<'de>> {
    guard: PoisonGuard<'p>,
    inner: T::EnumDecoder<'p>,
    read_discriminant: bool,
    read_variant: bool,
}

pub struct PoisonSomeParser<'p, 'de, T: 'p + Decoder<'de>> {
    guard: PoisonGuard<'p>,
    inner: T::SomeDecoder<'p>,
    read_some: bool,
}

impl<'de, T: Decoder<'de>> Decoder<'de> for PoisonParser<T> {
    type AnyDecoder<'p> = PoisonAnyParser<'p, 'de, T> where Self: 'p;
    type SeqDecoder<'p> = PoisonSeqParser<'p, 'de, T> where Self: 'p;
    type MapDecoder<'p> = PoisonMapParser<'p, 'de, T> where Self: 'p;
    type EntryDecoder<'p> = PoisonEntryParser<'p, 'de, T> where Self: 'p;
    type EnumDecoder<'p> = PoisonEnumParser<'p, 'de, T> where Self: 'p;
    type SomeDecoder<'p> = PoisonSomeParser<'p,'de,T> where Self: 'p;
}

impl<'p, 'de, T: Decoder<'de>> PoisonAnyParser<'p, 'de, T> {
    pub fn new(state: &'p mut PoisonState, inner: T::AnyDecoder<'p>) -> Self {
        PoisonAnyParser {
            guard: PoisonGuard::new(state, "Did not call AnyParser::decode"),
            inner,
        }
    }
}

fn annotate_view<'p, 'de, T: Decoder<'de>>(
    state: &'p mut PoisonState,
    view: DecoderView<'p, 'de, T>,
) -> DecoderView<'p, 'de, PoisonParser<T>> {
    match view {
        DecoderView::Primitive(x) => DecoderView::Primitive(x),
        DecoderView::String(x) => DecoderView::String(x),
        DecoderView::Bytes(x) => DecoderView::Bytes(x),
        DecoderView::None => DecoderView::None,
        DecoderView::Some(x) => DecoderView::Some(PoisonSomeParser::new(state, x)),
        DecoderView::Seq(x) => DecoderView::Seq(PoisonSeqParser::new(state, x)),
        DecoderView::Map(x) => DecoderView::Map(PoisonMapParser::new(state, x)),
        DecoderView::Enum(x) => DecoderView::Enum(PoisonEnumParser::new(state, x)),
    }
}

impl<'p, 'de, T: Decoder<'de>> AnyDecoder<'p, 'de, PoisonParser<T>> for PoisonAnyParser<'p, 'de, T> {
    fn decode(mut self, hint: DecodeHint) -> anyhow::Result<DecoderView<'p, 'de, PoisonParser<T>>> {
        self.guard.check()?;
        let inner = self.inner.decode(hint)?;
        let state = self.guard.defuse();
        Ok(annotate_view(state, inner))
    }
}

impl<'p, 'de, T: Decoder<'de>> PoisonSeqParser<'p, 'de, T> {
    pub fn new(state: &'p mut PoisonState, inner: T::SeqDecoder<'p>) -> Self {
        PoisonSeqParser {
            guard: PoisonGuard::new(state, "Did not finish consuming seq"),
            inner,
        }
    }
}

impl<'p, 'de, T: Decoder<'de>> SeqDecoder<'p, 'de, PoisonParser<T>> for PoisonSeqParser<'p, 'de, T> {
    fn decode_next<'p2>(&'p2 mut self) -> anyhow::Result<Option<PoisonAnyParser<'p2, 'de, T>>> {
        self.guard.check()?;
        if let Some(result) = self.inner.decode_next()? {
            let state = self.guard.state();
            Ok(Some(PoisonAnyParser::new(state, result)))
        } else {
            self.guard.defuse();
            Ok(None)
        }
    }
}

impl<'p, 'de, T: Decoder<'de>> PoisonMapParser<'p, 'de, T> {
    pub fn new(state: &'p mut PoisonState, inner: T::MapDecoder<'p>) -> Self {
        PoisonMapParser {
            guard: PoisonGuard::new(state, "Did not finish consuming map"),
            inner,
        }
    }
}

impl<'p, 'de, T: Decoder<'de>> MapDecoder<'p, 'de, PoisonParser<T>> for PoisonMapParser<'p, 'de, T> {
    fn decode_next<'p2>(&'p2 mut self) -> anyhow::Result<Option<PoisonEntryParser<'p2, 'de, T>>> {
        self.guard.check()?;
        if let Some(result) = self.inner.decode_next()? {
            let state = self.guard.state();
            Ok(Some(PoisonEntryParser::new(state, result)))
        } else {
            self.guard.defuse();
            Ok(None)
        }
    }
}

impl<'p, 'de, T: Decoder<'de>> PoisonEntryParser<'p, 'de, T> {
    pub fn new(state: &'p mut PoisonState, inner: T::EntryDecoder<'p>) -> Self {
        PoisonEntryParser {
            guard: PoisonGuard::new(state, "Did not finish consuming entry"),
            inner,
            read_key: false,
            read_value: false,
        }
    }
}

impl<'p, 'de, T: Decoder<'de>> EntryDecoder<'p, 'de, PoisonParser<T>>
    for PoisonEntryParser<'p, 'de, T>
{
    fn decode_key<'p2>(&'p2 mut self) -> anyhow::Result<PoisonAnyParser<'p2, 'de, T>> {
        if self.read_key {
            return Err(PoisonError("already read key").into());
        }
        self.read_key = true;
        self.guard.check()?;
        let result = self.inner.decode_key()?;
        let state = self.guard.state();
        Ok(PoisonAnyParser::new(state, result))
    }

    fn decode_value<'p2>(&'p2 mut self) -> anyhow::Result<PoisonAnyParser<'p2, 'de, T>> {
        if !self.read_key || self.read_value {
            return Err(PoisonError("already read value").into());
        }
        self.read_value = true;
        self.guard.check()?;
        let result = self.inner.decode_value()?;
        let state = self.guard.state();
        Ok(PoisonAnyParser::new(state, result))
    }

    fn decode_end(mut self) -> anyhow::Result<()> {
        if !self.read_value {
            return Err(PoisonError("did not read value").into());
        }
        self.guard.check()?;
        self.guard.defuse();
        Ok(())
    }
}

impl<'p, 'de, T: Decoder<'de>> PoisonEnumParser<'p, 'de, T> {
    pub fn new(state: &'p mut PoisonState, inner: T::EnumDecoder<'p>) -> Self {
        PoisonEnumParser {
            guard: PoisonGuard::new(state, "Did not finish reading enum"),
            inner,
            read_discriminant: false,
            read_variant: false,
        }
    }
}

impl<'p, 'de, T: Decoder<'de>> EnumDecoder<'p, 'de, PoisonParser<T>>
    for PoisonEnumParser<'p, 'de, T>
{
    fn decode_discriminant<'p2>(&'p2 mut self) -> anyhow::Result<PoisonAnyParser<'p2, 'de, T>> {
        println!("Reading discriminant");
        if self.read_discriminant {
            return Err(PoisonError("already read discriminant").into());
        }
        self.read_discriminant = true;
        self.guard.check()?;
        let result = self.inner.decode_discriminant()?;
        let state = self.guard.state();
        Ok(PoisonAnyParser::new(state, result))
    }

    fn decode_variant<'p2>(
        &'p2 mut self,
        hint: DecodeVariantHint,
    ) -> anyhow::Result<DecoderView<'p2, 'de, PoisonParser<T>>> {
        println!("Reading variant");
        if !self.read_discriminant {
            return Err(PoisonError("did not read discriminant").into());
        }
        if self.read_variant {
            return Err(PoisonError("already read variant").into());
        }
        self.read_variant = true;
        self.guard.check()?;
        let result = self.inner.decode_variant(hint)?;
        let state = self.guard.state();
        Ok(annotate_view(state, result))
    }

    fn decode_end(mut self) -> anyhow::Result<()> {
        println!("Ending enum");
        if !self.read_variant {
            return Err(PoisonError("Did not read variant").into());
        }
        self.guard.check()?;
        self.inner.decode_end()?;
        self.guard.defuse();
        Ok(())
    }
}

impl<'p, 'de, T: Decoder<'de>> PoisonSomeParser<'p, 'de, T> {
    pub fn new(state: &'p mut PoisonState, inner: T::SomeDecoder<'p>) -> Self {
        PoisonSomeParser {
            guard: PoisonGuard::new(state, "Did not finish reading some"),
            inner,
            read_some: false,
        }
    }
}

impl<'p, 'de, T: Decoder<'de>> SomeDecoder<'p, 'de, PoisonParser<T>>
    for PoisonSomeParser<'p, 'de, T>
{
    fn decode_some<'p2>(&'p2 mut self) -> anyhow::Result<PoisonAnyParser<'p2, 'de, T>> {
        if self.read_some {
            return Err(PoisonError("already read some").into());
        }
        self.read_some = true;
        self.guard.check()?;
        let inner = self.inner.decode_some()?;
        Ok(PoisonAnyParser::new(self.guard.state(), inner))
    }

    fn decode_end(mut self) -> anyhow::Result<()> {
        if !self.read_some {
            return Err(PoisonError("did not read some").into());
        }
        self.guard.check()?;
        self.inner.decode_end()?;
        self.guard.defuse();
        Ok(())
    }
}
