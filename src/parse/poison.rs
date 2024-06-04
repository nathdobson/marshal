use std::error::Error;
use std::fmt::{Display, Formatter};
use std::marker::PhantomData;

use crate::parse::{
    AnyParser, EntryParser, EnumParser, MapParser, NewtypeParser, ParseHint, ParseVariantHint,
    Parser, ParserView, SeqParser, SomeParser,
};

pub struct PoisonParser<T>(PhantomData<T>);

pub struct PoisonState {
    poisoned: bool,
}

impl PoisonState {
    pub fn new() -> PoisonState {
        PoisonState { poisoned: false }
    }
}

#[derive(Debug)]
pub struct PoisonError;

impl Display for PoisonError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "parser dropped before being fully consumed")
    }
}

impl Error for PoisonError {}

pub struct PoisonGuard<'p> {
    state: Option<&'p mut PoisonState>,
}

impl PoisonState {
    pub fn check(self) -> Result<(), PoisonError> {
        if self.poisoned {
            Err(PoisonError)
        } else {
            Ok(())
        }
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
    pub fn check(&self) -> Result<(), PoisonError> {
        if self.state.as_ref().unwrap().poisoned {
            Err(PoisonError)
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

pub struct PoisonSomeParser<'p, 'de, T: 'p + Parser<'de>> {
    guard: PoisonGuard<'p>,
    inner: T::SomeParser<'p>,
}

pub struct PoisonNewtypeParser<'p, 'de, T: 'p + Parser<'de>> {
    guard: PoisonGuard<'p>,
    inner: T::NewtypeParser<'p>,
}

impl<'de, T: Parser<'de>> Parser<'de> for PoisonParser<T> {
    type AnyParser<'p> = PoisonAnyParser<'p, 'de, T> where Self: 'p;
    type SeqParser<'p> = PoisonSeqParser<'p, 'de, T> where Self: 'p;
    type MapParser<'p> = PoisonMapParser<'p, 'de, T> where Self: 'p;
    type EntryParser<'p> = PoisonEntryParser<'p, 'de, T> where Self: 'p;
    type EnumParser<'p> = PoisonEnumParser<'p, 'de, T> where Self: 'p;
    type SomeParser<'p> = PoisonSomeParser<'p,'de,T> where Self: 'p;
    type NewtypeParser<'p> = PoisonNewtypeParser<'p,'de,T> where Self: 'p;
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
        ParserView::Primitive(x) => ParserView::Primitive(x),
        ParserView::String(x) => ParserView::String(x),
        ParserView::Bytes(x) => ParserView::Bytes(x),
        ParserView::None => ParserView::None,
        ParserView::Some(x) => ParserView::Some(PoisonSomeParser::new(state, x)),
        ParserView::Newtype(x) => ParserView::Newtype(PoisonNewtypeParser::new(state, x)),
        ParserView::Seq(x) => ParserView::Seq(PoisonSeqParser::new(state, x)),
        ParserView::Map(x) => ParserView::Map(PoisonMapParser::new(state, x)),
        ParserView::Enum(x) => ParserView::Enum(PoisonEnumParser::new(state, x)),
    }
}

impl<'p, 'de, T: Parser<'de>> AnyParser<'p, 'de, PoisonParser<T>> for PoisonAnyParser<'p, 'de, T> {
    fn parse(mut self, hint: ParseHint) -> anyhow::Result<ParserView<'p, 'de, PoisonParser<T>>> {
        self.guard.check()?;
        let inner = self.inner.parse(hint)?;
        let state = self.guard.defuse();
        Ok(annotate_view(state, inner))
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
    fn parse_next<'p2>(&'p2 mut self) -> anyhow::Result<Option<PoisonAnyParser<'p2, 'de, T>>> {
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
    fn parse_next<'p2>(&'p2 mut self) -> anyhow::Result<Option<PoisonEntryParser<'p2, 'de, T>>> {
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
    fn parse_key<'p2>(&'p2 mut self) -> anyhow::Result<PoisonAnyParser<'p2, 'de, T>> {
        self.guard.check()?;
        let result = self.inner.parse_key()?;
        let state = self.guard.state();
        Ok(PoisonAnyParser::new(state, result))
    }

    fn parse_value<'p2>(&'p2 mut self) -> anyhow::Result<PoisonAnyParser<'p2, 'de, T>> {
        self.guard.check()?;
        let result = self.inner.parse_value()?;
        let state = self.guard.defuse();
        Ok(PoisonAnyParser::new(state, result))
    }

    fn parse_end(self) -> anyhow::Result<()> {
        todo!()
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
    fn parse_discriminant<'p2>(&'p2 mut self) -> anyhow::Result<PoisonAnyParser<'p2, 'de, T>> {
        self.guard.check()?;
        let result = self.inner.parse_discriminant()?;
        let state = self.guard.state();
        Ok(PoisonAnyParser::new(state, result))
    }

    fn parse_variant<'p2>(
        &'p2 mut self,
        hint: ParseVariantHint,
    ) -> anyhow::Result<ParserView<'p2, 'de, PoisonParser<T>>> {
        self.guard.check()?;
        let result = self.inner.parse_variant(hint)?;
        let state = self.guard.defuse();
        Ok(annotate_view(state, result))
    }

    fn parse_end(self) -> anyhow::Result<()> {
        todo!()
    }
}

impl<'p, 'de, T: Parser<'de>> PoisonSomeParser<'p, 'de, T> {
    pub fn new(state: &'p mut PoisonState, inner: T::SomeParser<'p>) -> Self {
        PoisonSomeParser {
            guard: PoisonGuard::new(state),
            inner,
        }
    }
}

impl<'p, 'de, T: Parser<'de>> SomeParser<'p, 'de, PoisonParser<T>>
    for PoisonSomeParser<'p, 'de, T>
{
    fn parse_some<'p2>(&'p2 mut self) -> anyhow::Result<PoisonAnyParser<'p2, 'de, T>> {
        todo!()
    }

    fn parse_end(self) -> anyhow::Result<()> {
        todo!()
    }
}

impl<'p, 'de, T: Parser<'de>> PoisonNewtypeParser<'p, 'de, T> {
    pub fn new(state: &'p mut PoisonState, inner: T::NewtypeParser<'p>) -> Self {
        PoisonNewtypeParser {
            guard: PoisonGuard::new(state),
            inner,
        }
    }
}

impl<'p, 'de, T: Parser<'de>> NewtypeParser<'p, 'de, PoisonParser<T>>
    for PoisonNewtypeParser<'p, 'de, T>
{
    fn parse_newtype<'p2>(&'p2 mut self) -> anyhow::Result<PoisonAnyParser<'p2, 'de, T>> {
        todo!()
    }

    fn parse_end(self) -> anyhow::Result<()> {
        todo!()
    }
}
