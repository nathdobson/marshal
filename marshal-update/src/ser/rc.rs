use std::sync;
use std::sync::Arc;

use marshal::context::Context;
use marshal::encode::{AnyEncoder, AnyGenEncoder, Encoder, GenEncoder};
use marshal::reexports::marshal_pointer::DerefRaw;
use marshal::ser::Serialize;

use crate::ser::{SerializeStream, SerializeUpdate};

impl<T: ?Sized + Sync + Send> SerializeStream for Arc<T> {
    type Stream = sync::Weak<T>;
    fn start_stream(&self, _ctx: Context) -> anyhow::Result<Self::Stream> {
        Ok(Arc::downgrade(self))
    }
}

impl<T: ?Sized + Sync + Send, E: GenEncoder> SerializeUpdate<E> for Arc<T>
where
    Arc<T>: Serialize<E>,
{
    fn serialize_update<'w, 'en>(
        &self,
        stream: &mut Self::Stream,
        e: AnyGenEncoder<'w, 'en, E>,
        ctx: Context,
    ) -> anyhow::Result<()> {
        let m = if stream.deref_raw() as *const () != self.deref_raw() as *const () {
            *stream = Arc::downgrade(self);
            Some(self)
        } else {
            None
        };
        m.serialize(e, ctx)
    }
}

impl<T: ?Sized + Sync + Send> SerializeStream for sync::Weak<T> {
    type Stream = sync::Weak<T>;
    fn start_stream(&self, _ctx: Context) -> anyhow::Result<Self::Stream> {
        Ok(self.clone())
    }
}

impl<T: ?Sized + Sync + Send, E: GenEncoder> SerializeUpdate<E> for sync::Weak<T>
where
    sync::Weak<T>: Serialize<E>,
{
    fn serialize_update<'w, 'en>(
        &self,
        stream: &mut Self::Stream,
        e: AnyGenEncoder<'w, 'en, E>,
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
