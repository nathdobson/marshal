use crate::de::DeserializeUpdate;
use crate::ser::{SerializeStream, SerializeUpdate};
use marshal::context::Context;
use marshal::de::Deserialize;
use marshal::decode::{AnyDecoder, DecodeHint, Decoder};
use marshal::encode::{AnyEncoder, Encoder};
use marshal::ser::Serialize;
use std::borrow::Borrow;
use std::fmt::{Debug, Formatter};
use std::ops::Deref;

pub struct PushVec<T> {
    inner: Vec<T>,
}

pub struct PushVecStream {
    len: usize,
}

impl<T> PushVec<T> {
    pub fn new() -> Self {
        PushVec { inner: vec![] }
    }
    pub fn push(&mut self, v: T) {
        self.inner.push(v);
    }
}

impl<E: Encoder, T: Serialize<E>> Serialize<E> for PushVec<T> {
    fn serialize<'w, 'en>(&self, e: AnyEncoder<'w, 'en, E>, ctx: Context) -> anyhow::Result<()> {
        self.inner.serialize(e, ctx)
    }
}

impl<D: Decoder, T: Deserialize<D>> Deserialize<D> for PushVec<T> {
    fn deserialize<'p, 'de>(d: AnyDecoder<'p, 'de, D>, ctx: Context) -> anyhow::Result<Self> {
        Ok(PushVec {
            inner: Vec::<T>::deserialize(d, ctx)?,
        })
    }
}

impl<T> SerializeStream for PushVec<T> {
    type Stream = PushVecStream;
    fn start_stream(&self, _ctx: Context) -> anyhow::Result<Self::Stream> {
        Ok(PushVecStream {
            len: self.inner.len(),
        })
    }
}

impl<E: Encoder, T: Serialize<E>> SerializeUpdate<E> for PushVec<T> {
    fn serialize_update(
        &self,
        stream: &mut Self::Stream,
        e: AnyEncoder<E>,
        ctx: Context,
    ) -> anyhow::Result<()> {
        self.inner[stream.len..].serialize(e, ctx)?;
        stream.len = self.inner.len();
        Ok(())
    }
}

impl<D: Decoder, T: Deserialize<D>> DeserializeUpdate<D> for PushVec<T> {
    fn deserialize_update<'p, 'de>(
        &mut self,
        d: AnyDecoder<'p, 'de, D>,
        mut ctx: Context,
    ) -> anyhow::Result<()> {
        let mut d = d.decode(DecodeHint::Seq)?.try_into_seq()?;
        if let Some(size) = d.exact_size() {
            self.inner.reserve(size);
        }
        while let Some(d) = d.decode_next()? {
            self.inner.push(T::deserialize(d, ctx.reborrow())?);
        }
        Ok(())
    }
}

impl<T> Deref for PushVec<T> {
    type Target = [T];
    fn deref(&self) -> &Self::Target {
        self.inner.deref()
    }
}

impl<T> AsRef<[T]> for PushVec<T> {
    fn as_ref(&self) -> &[T] {
        self.inner.as_ref()
    }
}

impl<T> Borrow<[T]> for PushVec<T> {
    fn borrow(&self) -> &[T] {
        self.inner.borrow()
    }
}

impl<T: Debug> Debug for PushVec<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        self.inner.fmt(f)
    }
}

impl<T> Default for PushVec<T> {
    fn default() -> Self {
        PushVec::new()
    }
}
