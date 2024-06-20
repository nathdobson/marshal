use crate::de::DeserializeUpdate;
use crate::ser::{SerializeStream, SerializeUpdate};
use crate::version::Version;
use marshal::context::Context;
use marshal::de::Deserialize;
use marshal::decode::{AnyDecoder, Decoder};
use marshal::encode::{AnyEncoder, Encoder};
use marshal::ser::Serialize;
use std::ops::{Deref, DerefMut};

pub struct Prim<T: ?Sized> {
    version: Version,
    inner: T,
}

impl<T: ?Sized> Prim<T> {
    pub fn new(inner: T) -> Self
    where
        T: Sized,
    {
        Prim {
            version: Version::new(),
            inner,
        }
    }
}

impl<T: ?Sized> Deref for Prim<T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<T: ?Sized> DerefMut for Prim<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.version.next();
        &mut self.inner
    }
}

pub struct PrimStream {
    version: Version,
}

impl<E: Encoder, T: ?Sized + Serialize<E>> Serialize<E> for Prim<T> {
    fn serialize(&self, e: AnyEncoder<'_, E>, ctx: &mut Context) -> anyhow::Result<()> {
        self.inner.serialize(e, ctx)
    }
}

impl<'de, D: Decoder<'de>, T: Deserialize<'de, D>> Deserialize<'de, D> for Prim<T> {
    fn deserialize<'p>(d: AnyDecoder<'p, 'de, D>, ctx: &mut Context) -> anyhow::Result<Self> {
        Ok(Prim::new(T::deserialize(d, ctx)?))
    }
}

impl<'de, D: Decoder<'de>, T: Deserialize<'de, D>> DeserializeUpdate<'de, D> for Prim<T> {
    fn deserialize_update<'p>(
        &mut self,
        d: AnyDecoder<'p, 'de, D>,
        ctx: &mut Context,
    ) -> anyhow::Result<()> {
        if let Some(x) = Option::<T>::deserialize(d, ctx)? {
            **self = x;
        }
        Ok(())
    }
}

impl<T: ?Sized> SerializeStream for Prim<T> {
    type Stream = PrimStream;

    fn start_stream(&self, _ctx: &mut Context) -> anyhow::Result<Self::Stream> {
        Ok(PrimStream {
            version: self.version,
        })
    }
}

impl<E: Encoder, T: Serialize<E>> SerializeUpdate<E> for Prim<T> {
    fn serialize_update(
        &self,
        stream: &mut Self::Stream,
        e: AnyEncoder<E>,
        ctx: &mut Context,
    ) -> anyhow::Result<()> {
        let m = if stream.version != self.version {
            stream.version = self.version;
            Some(&self.inner)
        } else {
            None
        };
        m.serialize(e, ctx)?;
        Ok(())
    }
}
