use std::sync;
use std::sync::Arc;

use crate::ser::{SerializeStream, SerializeUpdate};
use marshal::context::Context;
use marshal::encode::{AnyEncoder, Encoder};
use marshal::ser::Serialize;
use marshal_pointer::raw_any::DerefRaw;
use marshal_pointer::{Arcf, ArcfWeak};

impl<T: ?Sized + Sync + Send> SerializeStream for Arcf<T> {
    type Stream = ArcfWeak<T>;
    fn start_stream(&self, _ctx: Context) -> anyhow::Result<Self::Stream> {
        Ok(Arcf::downgrade(self))
    }
}

impl<T: ?Sized + Sync + Send, E: Encoder> SerializeUpdate<E> for Arcf<T>
where
    Arcf<T>: Serialize<E>,
{
    fn serialize_update<'w, 'en>(
        &self,
        stream: &mut Self::Stream,
        e: AnyEncoder<'w, 'en, E>,
        ctx: Context,
    ) -> anyhow::Result<()> {
        let m = if stream.deref_raw() as *const () != self.deref_raw() as *const () {
            *stream = Arcf::downgrade(self);
            Some(self)
        } else {
            None
        };
        m.serialize(e, ctx)
    }
}

impl<T: ?Sized + Sync + Send> SerializeStream for ArcfWeak<T> {
    type Stream = ArcfWeak<T>;
    fn start_stream(&self, _ctx: Context) -> anyhow::Result<Self::Stream> {
        Ok(self.clone())
    }
}

impl<T: ?Sized + Sync + Send, E: Encoder> SerializeUpdate<E> for ArcfWeak<T>
where
    ArcfWeak<T>: Serialize<E>,
{
    fn serialize_update<'w, 'en>(
        &self,
        stream: &mut Self::Stream,
        e: AnyEncoder<'w, 'en, E>,
        ctx: Context,
    ) -> anyhow::Result<()> {
        let m = if stream.deref_raw() as *const () != self.deref_raw() as *const () {
            *stream = self.clone();
            Some(self)
        } else {
            None
        };
        m.serialize(e, ctx)
    }
}
