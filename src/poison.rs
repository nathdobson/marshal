use crate::{
    AnyParser, EntryParser, EnumParser, MapParser, ParseHint, ParseVariantHint, Parser, ParserView,
    SeqParser,
};
use std::marker::PhantomData;

pub struct PoisonParser<T>(PhantomData<T>);

pub struct PoisonState {
    poisoned: bool,
}

pub enum PoisonError<E> {
    Poisoned,
    Other(E),
}

pub struct PoisonGuard<'p> {
    state: Option<&'p mut PoisonState>,
}

impl<E> From<E> for PoisonError<E> {
    fn from(value: E) -> Self {
        PoisonError::Other(value)
    }
}

impl<'p> PoisonGuard<'p> {
    pub fn new(state: &'p mut PoisonState) -> Self {
        PoisonGuard { state: Some(state) }
    }
    pub fn defuse(&mut self) -> &'p mut PoisonState {
        self.state.take().unwrap()
    }
    pub fn defuse_into(mut self) -> &'p mut PoisonState {
        self.state.take().unwrap()
    }
    pub fn check<E>(&self) -> Result<(), PoisonError<E>> {
        if self.state.as_ref().unwrap().poisoned {
            Err(PoisonError::Poisoned)
        } else {
            Ok(())
        }
    }
    pub fn state<'p2>(&'p2 mut self) -> &'p2 mut PoisonState {
        self.state.as_mut().unwrap()
    }
}

impl<'p> Drop for PoisonGuard<'p> {
    fn drop(&mut self) {
        if let Some(state) = self.state.take() {
            state.poisoned = true;
        }
    }
}

pub struct PoisonAnyParser<'p, 'de, T: 'p + Parser<'de>> {
    guard: PoisonGuard<'p>,
    inner: T::AnyParser<'p>,
}

pub struct PoisonSeqParser<'p, 'de, T: 'p + Parser<'de>> {
    guard: PoisonGuard<'p>,
    inner: T::SeqParser<'p>,
}

pub struct PoisonMapParser<'p, 'de, T: 'p + Parser<'de>> {
    guard: PoisonGuard<'p>,
    inner: T::MapParser<'p>,
}

pub struct PoisonEntryParser<'p, 'de, T: 'p + Parser<'de>> {
    guard: PoisonGuard<'p>,
    inner: T::EntryParser<'p>,
}

pub struct PoisonEnumParser<'p, 'de, T: 'p + Parser<'de>> {
    guard: PoisonGuard<'p>,
    inner: T::EnumParser<'p>,
}

impl<'de, T: Parser<'de>> Parser<'de> for PoisonParser<T> {
    type Error = PoisonError<T::Error>;
    type AnyParser<'p> = PoisonAnyParser<'p, 'de, T> where Self: 'p;
    type SeqParser<'p> = PoisonSeqParser<'p, 'de, T> where Self: 'p;
    type MapParser<'p> = PoisonMapParser<'p, 'de, T> where Self: 'p;
    type EntryParser<'p> = PoisonEntryParser<'p, 'de, T> where Self: 'p;
    type EnumParser<'p> = PoisonEnumParser<'p, 'de, T> where Self: 'p;
}

impl<'p, 'de, T: Parser<'de>> PoisonAnyParser<'p, 'de, T> {
    pub fn new(state: &'p mut PoisonState, inner: T::AnyParser<'p>) -> Self {
        PoisonAnyParser {
            guard: PoisonGuard::new(state),
            inner,
        }
    }
}

fn annotate_view<'p, 'de, T: Parser<'de>>(
    state: &'p mut PoisonState,
    view: ParserView<'p, 'de, T>,
) -> ParserView<'p, 'de, PoisonParser<T>> {
    match view {
        ParserView::Bool(x) => ParserView::Bool(x),
        ParserView::I64(x) => ParserView::I64(x),
        ParserView::U64(x) => ParserView::U64(x),
        ParserView::F64(x) => ParserView::F64(x),
        ParserView::Char(x) => ParserView::Char(x),
        ParserView::String(x) => ParserView::String(x),
        ParserView::Bytes(x) => ParserView::Bytes(x),
        ParserView::None => ParserView::None,
        ParserView::Some(x) => ParserView::Some(PoisonAnyParser::new(state, x)),
        ParserView::Unit => ParserView::Unit,
        ParserView::Newtype(x) => ParserView::Newtype(PoisonAnyParser::new(state, x)),
        ParserView::Seq(x) => ParserView::Seq(PoisonSeqParser::new(state, x)),
        ParserView::Map(x) => ParserView::Map(PoisonMapParser::new(state, x)),
        ParserView::Enum(x) => ParserView::Enum(PoisonEnumParser::new(state, x)),
    }
}

impl<'p, 'de, T: Parser<'de>> AnyParser<'p, 'de, PoisonParser<T>> for PoisonAnyParser<'p, 'de, T> {
    fn parse(
        mut self,
        hint: ParseHint,
    ) -> Result<ParserView<'p, 'de, PoisonParser<T>>, PoisonError<T::Error>> {
        self.guard.check()?;
        let inner = self.inner.parse(hint)?;
        let state = self.guard.defuse();
        Ok(annotate_view(state, inner))
    }

    fn is_human_readable(&self) -> bool {
        self.inner.is_human_readable()
    }
}

impl<'p, 'de, T: Parser<'de>> PoisonSeqParser<'p, 'de, T> {
    pub fn new(state: &'p mut PoisonState, inner: T::SeqParser<'p>) -> Self {
        PoisonSeqParser {
            guard: PoisonGuard::new(state),
            inner,
        }
    }
}

impl<'p, 'de, T: Parser<'de>> SeqParser<'p, 'de, PoisonParser<T>> for PoisonSeqParser<'p, 'de, T> {
    fn parse_next<'p2>(
        &'p2 mut self,
    ) -> Result<Option<PoisonAnyParser<'p2, 'de, T>>, PoisonError<T::Error>> {
        self.guard.check()?;
        if let Some(result) = self.inner.parse_next()? {
            let state = self.guard.state();
            Ok(Some(PoisonAnyParser::new(state, result)))
        } else {
            self.guard.defuse();
            Ok(None)
        }
    }
}

impl<'p, 'de, T: Parser<'de>> PoisonMapParser<'p, 'de, T> {
    pub fn new(state: &'p mut PoisonState, inner: T::MapParser<'p>) -> Self {
        PoisonMapParser {
            guard: PoisonGuard::new(state),
            inner,
        }
    }
}

impl<'p, 'de, T: Parser<'de>> MapParser<'p, 'de, PoisonParser<T>> for PoisonMapParser<'p, 'de, T> {
    fn parse_next<'p2>(
        &'p2 mut self,
    ) -> Result<Option<PoisonEntryParser<'p2, 'de, T>>, PoisonError<T::Error>> {
        self.guard.check()?;
        if let Some(result) = self.inner.parse_next()? {
            let state = self.guard.state();
            Ok(Some(PoisonEntryParser::new(state, result)))
        } else {
            self.guard.defuse();
            Ok(None)
        }
    }
}

impl<'p, 'de, T: Parser<'de>> PoisonEntryParser<'p, 'de, T> {
    pub fn new(state: &'p mut PoisonState, inner: T::EntryParser<'p>) -> Self {
        PoisonEntryParser {
            guard: PoisonGuard::new(state),
            inner,
        }
    }
}

impl<'p, 'de, T: Parser<'de>> EntryParser<'p, 'de, PoisonParser<T>>
    for PoisonEntryParser<'p, 'de, T>
{
    fn parse_key<'p2>(
        &'p2 mut self,
    ) -> Result<PoisonAnyParser<'p2, 'de, T>, PoisonError<T::Error>> {
        self.guard.check()?;
        let result = self.inner.parse_key()?;
        let state = self.guard.state();
        Ok(PoisonAnyParser::new(state, result))
    }

    fn parse_value(mut self) -> Result<PoisonAnyParser<'p, 'de, T>, PoisonError<T::Error>> {
        self.guard.check()?;
        let result = self.inner.parse_value()?;
        let state = self.guard.defuse();
        Ok(PoisonAnyParser::new(state, result))
    }
}

impl<'p, 'de, T: Parser<'de>> PoisonEnumParser<'p, 'de, T> {
    pub fn new(state: &'p mut PoisonState, inner: T::EnumParser<'p>) -> Self {
        PoisonEnumParser {
            guard: PoisonGuard::new(state),
            inner,
        }
    }
}

impl<'p, 'de, T: Parser<'de>> EnumParser<'p, 'de, PoisonParser<T>>
    for PoisonEnumParser<'p, 'de, T>
{
    fn parse_discriminant<'p2>(
        &'p2 mut self,
    ) -> Result<PoisonAnyParser<'p2, 'de, T>, PoisonError<T::Error>> {
        self.guard.check()?;
        let result = self.inner.parse_discriminant()?;
        let state = self.guard.state();
        Ok(PoisonAnyParser::new(state, result))
    }

    fn parse_variant<'p2>(
        &'p2 mut self,
        hint: ParseVariantHint,
    ) -> Result<ParserView<'p2, 'de, PoisonParser<T>>, PoisonError<T::Error>> {
        self.guard.check()?;
        let result = self.inner.parse_variant(hint)?;
        let state = self.guard.defuse();
        Ok(annotate_view(state, result))
    }
}
