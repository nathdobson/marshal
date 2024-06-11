use crate::encode::{
    AnyEncoder, Encoder, EntryEncoder, MapEncoder, SeqEncoder, SomeEncoder, StructEncoder,
    StructVariantEncoder, TupleEncoder, TupleStructEncoder, TupleVariantEncoder,
};
use crate::Primitive;
use std::error::Error;
use std::fmt::{Debug, Display, Formatter};
use std::marker::PhantomData;

pub struct PoisonEncoder<E>(PhantomData<E>);

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
    AnyEncoder,
    SomeEncoder,
    TupleEncoder,
    SeqEncoder,
    MapEncoder,
    TupleStructEncoder,
    StructEncoder,
    TupleVariantEncoder,
    StructVariantEncoder,
    SomeEncoderRewroteSome,
    SomeEncoderForgotSome,
    TupleEncoderTooLong,
    TupleEncoderTooShort,
    SeqEncoderTooLong,
    SeqEncoderTooShort,
    EntryEncoderRewroteSome,
    EntryEncoderForgotKey,
    EntryEncoderRewroteValue,
    EntryEncoderForgotValue,
    TupleStructEncoderTooLong,
    TupleStructEncoderTooShort,
    StructEncoderTooLong,
    TupleVariantEncoderTooLong,
    TupleVariantEncoderTooShort,
    StructVariantEncoderTooLong,
    StructVariantEncoderTooShort,
}

impl Display for PoisonError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "encoder not consumed properly: {:?}", self)
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

pub struct PoisonAnyEncoder<'w, E: 'w + Encoder> {
    guard: PoisonGuard<'w>,
    inner: E::AnyEncoder<'w>,
}

impl<'w, E: 'w + Encoder> PoisonAnyEncoder<'w, E> {
    pub fn new(poison: &'w mut PoisonState, inner: E::AnyEncoder<'w>) -> Self {
        PoisonAnyEncoder {
            guard: PoisonGuard::new(poison, PoisonError::AnyEncoder),
            inner,
        }
    }
}

pub struct PoisonSomeEncoder<'w, E: 'w + Encoder> {
    guard: PoisonGuard<'w>,
    inner: E::SomeEncoder<'w>,
    encoded_some: bool,
}

impl<'w, E: 'w + Encoder> PoisonSomeEncoder<'w, E> {
    pub fn new(poison: &'w mut PoisonState, inner: E::SomeEncoder<'w>) -> Self {
        PoisonSomeEncoder {
            guard: PoisonGuard::new(poison, PoisonError::SomeEncoder),
            inner,
            encoded_some: false,
        }
    }
}

pub struct PoisonTupleEncoder<'w, E: 'w + Encoder> {
    guard: PoisonGuard<'w>,
    inner: E::TupleEncoder<'w>,
    remaining: usize,
}

impl<'w, E: 'w + Encoder> PoisonTupleEncoder<'w, E> {
    pub fn new(poison: &'w mut PoisonState, inner: E::TupleEncoder<'w>, len: usize) -> Self {
        PoisonTupleEncoder {
            guard: PoisonGuard::new(poison, PoisonError::TupleEncoder),
            inner,
            remaining: len,
        }
    }
}

pub struct PoisonSeqEncoder<'w, E: 'w + Encoder> {
    guard: PoisonGuard<'w>,
    inner: E::SeqEncoder<'w>,
    remaining: Option<usize>,
}

impl<'w, E: 'w + Encoder> PoisonSeqEncoder<'w, E> {
    pub fn new(poison: &'w mut PoisonState, inner: E::SeqEncoder<'w>, len: Option<usize>) -> Self {
        PoisonSeqEncoder {
            guard: PoisonGuard::new(poison, PoisonError::SeqEncoder),
            inner,
            remaining: len,
        }
    }
}
pub struct PoisonMapEncoder<'w, E: 'w + Encoder> {
    guard: PoisonGuard<'w>,
    inner: E::MapEncoder<'w>,
    remaining: Option<usize>,
}

impl<'w, E: 'w + Encoder> PoisonMapEncoder<'w, E> {
    pub fn new(poison: &'w mut PoisonState, inner: E::MapEncoder<'w>, len: Option<usize>) -> Self {
        PoisonMapEncoder {
            guard: PoisonGuard::new(poison, PoisonError::MapEncoder),
            inner,
            remaining: len,
        }
    }
}
pub struct PoisonEntryEncoder<'w, E: 'w + Encoder> {
    guard: PoisonGuard<'w>,
    inner: E::EntryEncoder<'w>,
    encoded_key: bool,
    encoded_value: bool,
}

impl<'w, E: 'w + Encoder> PoisonEntryEncoder<'w, E> {
    pub fn new(poison: &'w mut PoisonState, inner: E::EntryEncoder<'w>) -> Self {
        PoisonEntryEncoder {
            guard: PoisonGuard::new(poison, PoisonError::AnyEncoder),
            inner,
            encoded_key: false,
            encoded_value: false,
        }
    }
}

pub struct PoisonTupleStructEncoder<'w, E: 'w + Encoder> {
    guard: PoisonGuard<'w>,
    inner: E::TupleStructEncoder<'w>,
    remaining: usize,
}

impl<'w, E: 'w + Encoder> PoisonTupleStructEncoder<'w, E> {
    pub fn new(poison: &'w mut PoisonState, inner: E::TupleStructEncoder<'w>, len: usize) -> Self {
        PoisonTupleStructEncoder {
            guard: PoisonGuard::new(poison, PoisonError::TupleStructEncoder),
            inner,
            remaining: len,
        }
    }
}

pub struct PoisonStructEncoder<'w, E: 'w + Encoder> {
    guard: PoisonGuard<'w>,
    inner: E::StructEncoder<'w>,
    remaining: usize,
}

impl<'w, E: 'w + Encoder> PoisonStructEncoder<'w, E> {
    pub fn new(poison: &'w mut PoisonState, inner: E::StructEncoder<'w>, len: usize) -> Self {
        PoisonStructEncoder {
            guard: PoisonGuard::new(poison, PoisonError::StructEncoder),
            inner,
            remaining: len,
        }
    }
}

pub struct PoisonTupleVariantEncoder<'w, E: 'w + Encoder> {
    guard: PoisonGuard<'w>,
    inner: E::TupleVariantEncoder<'w>,
    remaining: usize,
}

impl<'w, E: 'w + Encoder> PoisonTupleVariantEncoder<'w, E> {
    pub fn new(poison: &'w mut PoisonState, inner: E::TupleVariantEncoder<'w>, len: usize) -> Self {
        PoisonTupleVariantEncoder {
            guard: PoisonGuard::new(poison, PoisonError::TupleVariantEncoder),
            inner,
            remaining: len,
        }
    }
}

pub struct PoisonStructVariantEncoder<'w, E: 'w + Encoder> {
    guard: PoisonGuard<'w>,
    inner: E::StructVariantEncoder<'w>,
    remaining: usize,
}

impl<'w, E: 'w + Encoder> PoisonStructVariantEncoder<'w, E> {
    pub fn new(
        poison: &'w mut PoisonState,
        inner: E::StructVariantEncoder<'w>,
        len: usize,
    ) -> Self {
        PoisonStructVariantEncoder {
            guard: PoisonGuard::new(poison, PoisonError::StructVariantEncoder),
            inner,
            remaining: len,
        }
    }
}

impl<E: Encoder> Encoder for PoisonEncoder<E> {
    type AnyEncoder<'w> = PoisonAnyEncoder<'w,E> where Self: 'w;
    type SomeEncoder<'w> = PoisonSomeEncoder<'w,E> where Self: 'w;
    type TupleEncoder<'w> = PoisonTupleEncoder<'w,E> where Self: 'w;
    type SeqEncoder<'w> = PoisonSeqEncoder<'w,E> where Self: 'w;
    type MapEncoder<'w> = PoisonMapEncoder<'w,E> where Self: 'w;
    type EntryEncoder<'w> = PoisonEntryEncoder<'w,E> where Self: 'w;
    type TupleStructEncoder<'w> = PoisonTupleStructEncoder<'w,E> where Self: 'w;
    type StructEncoder<'w> = PoisonStructEncoder<'w,E> where Self: 'w;
    type TupleVariantEncoder<'w> = PoisonTupleVariantEncoder<'w,E> where Self: 'w;
    type StructVariantEncoder<'w> = PoisonStructVariantEncoder<'w,E> where Self: 'w;
}

impl<'w, E: 'w + Encoder> AnyEncoder<'w, PoisonEncoder<E>> for PoisonAnyEncoder<'w, E> {
    fn encode_prim(mut self, prim: Primitive) -> anyhow::Result<()> {
        self.guard.check()?;
        self.inner.encode_prim(prim)?;
        self.guard.defuse();
        Ok(())
    }

    fn encode_str(mut self, s: &str) -> anyhow::Result<()> {
        self.guard.check()?;
        self.inner.encode_str(s)?;
        self.guard.defuse();
        Ok(())
    }

    fn encode_bytes(mut self, s: &[u8]) -> anyhow::Result<()> {
        self.guard.check()?;
        self.inner.encode_bytes(s)?;
        self.guard.defuse();
        Ok(())
    }

    fn encode_none(mut self) -> anyhow::Result<()> {
        self.guard.check()?;
        self.inner.encode_none()?;
        self.guard.defuse();
        Ok(())
    }

    fn encode_some(mut self) -> anyhow::Result<<PoisonEncoder<E> as Encoder>::SomeEncoder<'w>> {
        self.guard.check()?;
        let inner = self.inner.encode_some()?;
        Ok(PoisonSomeEncoder::new(self.guard.defuse_into(), inner))
    }

    fn encode_unit_struct(mut self, name: &'static str) -> anyhow::Result<()> {
        self.guard.check()?;
        self.inner.encode_unit_struct(name)?;
        self.guard.defuse();
        Ok(())
    }

    fn encode_tuple_struct(
        mut self,
        name: &'static str,
        len: usize,
    ) -> anyhow::Result<<PoisonEncoder<E> as Encoder>::TupleStructEncoder<'w>> {
        self.guard.check()?;
        let inner = self.inner.encode_tuple_struct(name, len)?;
        Ok(PoisonTupleStructEncoder::new(
            self.guard.defuse_into(),
            inner,
            len,
        ))
    }

    fn encode_struct(
        mut self,
        name: &'static str,
        fields: &'static [&'static str],
    ) -> anyhow::Result<<PoisonEncoder<E> as Encoder>::StructEncoder<'w>> {
        self.guard.check()?;
        let inner = self.inner.encode_struct(name, fields)?;
        Ok(PoisonStructEncoder::new(
            self.guard.defuse_into(),
            inner,
            fields.len(),
        ))
    }

    fn encode_unit_variant(
        mut self,
        name: &'static str,
        variants: &'static [&'static str],
        variant_index: usize,
    ) -> anyhow::Result<()> {
        self.guard.check()?;
        self.inner
            .encode_unit_variant(name, variants, variant_index)?;
        self.guard.defuse();
        Ok(())
    }

    fn encode_tuple_variant(
        mut self,
        name: &'static str,
        variants: &'static [&'static str],
        variant_index: usize,
        len: usize,
    ) -> anyhow::Result<<PoisonEncoder<E> as Encoder>::TupleVariantEncoder<'w>> {
        self.guard.check()?;
        let inner = self
            .inner
            .encode_tuple_variant(name, variants, variant_index, len)?;
        Ok(PoisonTupleVariantEncoder::new(
            self.guard.defuse_into(),
            inner,
            len,
        ))
    }

    fn encode_struct_variant(
        mut self,
        name: &'static str,
        variants: &'static [&'static str],
        variant_index: usize,
        fields: &'static [&'static str],
    ) -> anyhow::Result<<PoisonEncoder<E> as Encoder>::StructVariantEncoder<'w>> {
        self.guard.check()?;
        let inner = self
            .inner
            .encode_struct_variant(name, variants, variant_index, fields)?;
        Ok(PoisonStructVariantEncoder::new(
            self.guard.defuse_into(),
            inner,
            fields.len(),
        ))
    }

    fn encode_seq(
        mut self,
        len: Option<usize>,
    ) -> anyhow::Result<<PoisonEncoder<E> as Encoder>::SeqEncoder<'w>> {
        self.guard.check()?;
        let inner = self.inner.encode_seq(len)?;
        Ok(PoisonSeqEncoder::new(self.guard.defuse_into(), inner, len))
    }

    fn encode_tuple(
        mut self,
        len: usize,
    ) -> anyhow::Result<<PoisonEncoder<E> as Encoder>::TupleEncoder<'w>> {
        self.guard.check()?;
        let inner = self.inner.encode_tuple(len)?;
        Ok(PoisonTupleEncoder::new(
            self.guard.defuse_into(),
            inner,
            len,
        ))
    }

    fn encode_map(
        mut self,
        len: Option<usize>,
    ) -> anyhow::Result<<PoisonEncoder<E> as Encoder>::MapEncoder<'w>> {
        self.guard.check()?;
        let inner = self.inner.encode_map(len)?;
        Ok(PoisonMapEncoder::new(self.guard.defuse_into(), inner, len))
    }
}

impl<'w, E: 'w + Encoder> SomeEncoder<'w, PoisonEncoder<E>> for PoisonSomeEncoder<'w, E> {
    fn encode_some(&mut self) -> anyhow::Result<<PoisonEncoder<E> as Encoder>::AnyEncoder<'_>> {
        self.guard.check()?;
        if self.encoded_some {
            return Err(PoisonError::SomeEncoderRewroteSome.into());
        }
        self.encoded_some = true;
        let inner = self.inner.encode_some()?;
        Ok(PoisonAnyEncoder::new(self.guard.state(), inner))
    }

    fn end(mut self) -> anyhow::Result<()> {
        self.guard.check()?;
        if !self.encoded_some {
            return Err(PoisonError::SomeEncoderForgotSome.into());
        }
        self.guard.defuse();
        self.inner.end()?;
        Ok(())
    }
}
impl<'w, E: 'w + Encoder> TupleEncoder<'w, PoisonEncoder<E>> for PoisonTupleEncoder<'w, E> {
    fn encode_element(&mut self) -> anyhow::Result<<PoisonEncoder<E> as Encoder>::AnyEncoder<'_>> {
        self.guard.check()?;
        self.remaining = self
            .remaining
            .checked_sub(1)
            .ok_or(PoisonError::TupleEncoderTooLong)?;
        let inner = self.inner.encode_element()?;
        Ok(PoisonAnyEncoder::new(self.guard.state(), inner))
    }

    fn end(mut self) -> anyhow::Result<()> {
        self.guard.check()?;
        if self.remaining != 0 {
            return Err(PoisonError::TupleEncoderTooShort)?;
        }
        self.inner.end()?;
        self.guard.defuse();
        Ok(())
    }
}
impl<'w, E: 'w + Encoder> SeqEncoder<'w, PoisonEncoder<E>> for PoisonSeqEncoder<'w, E> {
    fn encode_element(&mut self) -> anyhow::Result<<PoisonEncoder<E> as Encoder>::AnyEncoder<'_>> {
        self.guard.check()?;
        if let Some(remaining) = &mut self.remaining {
            *remaining = remaining
                .checked_sub(1)
                .ok_or(PoisonError::SeqEncoderTooLong)?;
        }
        let inner = self.inner.encode_element()?;
        Ok(PoisonAnyEncoder::new(self.guard.state(), inner))
    }

    fn end(mut self) -> anyhow::Result<()> {
        self.guard.check()?;
        if let Some(remaining) = self.remaining {
            if remaining != 0 {
                return Err(PoisonError::SeqEncoderTooShort.into());
            }
        }
        self.inner.end()?;
        self.guard.defuse();
        Ok(())
    }
}
impl<'w, E: 'w + Encoder> MapEncoder<'w, PoisonEncoder<E>> for PoisonMapEncoder<'w, E> {
    fn encode_entry(&mut self) -> anyhow::Result<<PoisonEncoder<E> as Encoder>::EntryEncoder<'_>> {
        self.guard.check()?;
        if let Some(remaining) = &mut self.remaining {
            *remaining = remaining
                .checked_sub(1)
                .ok_or(PoisonError::SeqEncoderTooLong)?;
        }
        let inner = self.inner.encode_entry()?;
        Ok(PoisonEntryEncoder::new(self.guard.state(), inner))
    }

    fn end(mut self) -> anyhow::Result<()> {
        self.guard.check()?;
        if let Some(remaining) = self.remaining {
            if remaining != 0 {
                return Err(PoisonError::SeqEncoderTooShort.into());
            }
        }
        self.inner.end()?;
        self.guard.defuse();
        Ok(())
    }
}
impl<'w, E: 'w + Encoder> EntryEncoder<'w, PoisonEncoder<E>> for PoisonEntryEncoder<'w, E> {
    fn encode_key(&mut self) -> anyhow::Result<<PoisonEncoder<E> as Encoder>::AnyEncoder<'_>> {
        self.guard.check()?;
        if self.encoded_key {
            return Err(PoisonError::EntryEncoderRewroteSome.into());
        }
        self.encoded_key = true;
        let inner = self.inner.encode_key()?;
        Ok(PoisonAnyEncoder::new(self.guard.state(), inner))
    }

    fn encode_value(&mut self) -> anyhow::Result<<PoisonEncoder<E> as Encoder>::AnyEncoder<'_>> {
        self.guard.check()?;
        if !self.encoded_key {
            return Err(PoisonError::EntryEncoderForgotKey.into());
        }
        if self.encoded_value {
            return Err(PoisonError::EntryEncoderRewroteValue.into());
        }
        self.encoded_value = true;
        let inner = self.inner.encode_value()?;
        Ok(PoisonAnyEncoder::new(self.guard.state(), inner))
    }

    fn end(mut self) -> anyhow::Result<()> {
        self.guard.check()?;
        if !self.encoded_value {
            return Err(PoisonError::EntryEncoderForgotValue.into());
        }
        self.inner.end()?;
        self.guard.defuse();
        Ok(())
    }
}
impl<'w, E: 'w + Encoder> TupleStructEncoder<'w, PoisonEncoder<E>>
    for PoisonTupleStructEncoder<'w, E>
{
    fn encode_field(&mut self) -> anyhow::Result<<PoisonEncoder<E> as Encoder>::AnyEncoder<'_>> {
        self.guard.check()?;
        self.remaining = self
            .remaining
            .checked_sub(1)
            .ok_or(PoisonError::TupleStructEncoderTooLong)?;
        let inner = self.inner.encode_field()?;
        Ok(PoisonAnyEncoder::new(self.guard.state(), inner))
    }

    fn end(mut self) -> anyhow::Result<()> {
        self.guard.check()?;
        if self.remaining != 0 {
            return Err(PoisonError::TupleStructEncoderTooShort.into());
        }
        self.inner.end()?;
        self.guard.defuse();
        Ok(())
    }
}
impl<'w, E: 'w + Encoder> StructEncoder<'w, PoisonEncoder<E>> for PoisonStructEncoder<'w, E> {
    fn encode_field(&mut self) -> anyhow::Result<<PoisonEncoder<E> as Encoder>::AnyEncoder<'_>> {
        self.guard.check()?;
        self.remaining = self
            .remaining
            .checked_sub(1)
            .ok_or(PoisonError::StructEncoderTooLong)?;
        let inner = self.inner.encode_field()?;
        Ok(PoisonAnyEncoder::new(self.guard.state(), inner))
    }

    fn end(mut self) -> anyhow::Result<()> {
        self.guard.check()?;
        if self.remaining != 0 {
            return Err(PoisonError::TupleStructEncoderTooShort.into());
        }
        self.inner.end()?;
        self.guard.defuse();
        Ok(())
    }
}
impl<'w, E: 'w + Encoder> TupleVariantEncoder<'w, PoisonEncoder<E>>
    for PoisonTupleVariantEncoder<'w, E>
{
    fn encode_field(&mut self) -> anyhow::Result<<PoisonEncoder<E> as Encoder>::AnyEncoder<'_>> {
        self.guard.check()?;
        self.remaining = self
            .remaining
            .checked_sub(1)
            .ok_or(PoisonError::TupleVariantEncoderTooLong)?;
        let inner = self.inner.encode_field()?;
        Ok(PoisonAnyEncoder::new(self.guard.state(), inner))
    }

    fn end(mut self) -> anyhow::Result<()> {
        self.guard.check()?;
        if self.remaining != 0 {
            return Err(PoisonError::TupleVariantEncoderTooShort.into());
        }
        self.inner.end()?;
        self.guard.defuse();
        Ok(())
    }
}
impl<'w, E: 'w + Encoder> StructVariantEncoder<'w, PoisonEncoder<E>>
    for PoisonStructVariantEncoder<'w, E>
{
    fn encode_field(&mut self) -> anyhow::Result<<PoisonEncoder<E> as Encoder>::AnyEncoder<'_>> {
        self.guard.check()?;
        self.remaining = self
            .remaining
            .checked_sub(1)
            .ok_or(PoisonError::StructVariantEncoderTooLong)?;
        let inner = self.inner.encode_field()?;
        Ok(PoisonAnyEncoder::new(self.guard.state(), inner))
    }

    fn end(mut self) -> anyhow::Result<()> {
        self.guard.check()?;
        if self.remaining != 0 {
            return Err(PoisonError::StructVariantEncoderTooShort.into());
        }
        self.inner.end()?;
        self.guard.defuse();
        Ok(())
    }
}
