use crate::ser::{SerializeStream, SerializeUpdate};
use marshal::context::Context;
use marshal::encode::{AnyEncoder, Encoder};
use marshal::reexports::marshal_pointer::DerefRaw;
use marshal::ser::Serialize;
use std::sync;
use std::sync::Arc;

impl<T: ?Sized> SerializeStream for Arc<T> {
    type Stream = sync::Weak<T>;
    fn start_stream(&self, _ctx: &mut Context) -> anyhow::Result<Self::Stream> {
        Ok(Arc::downgrade(self))
    }
}

impl<T: ?Sized, E: Encoder> SerializeUpdate<E> for Arc<T>
where
    Arc<T>: Serialize<E>,
{
    fn serialize_update(
        &self,
        stream: &mut Self::Stream,
        e: AnyEncoder<E>,
        ctx: &mut Context,
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
