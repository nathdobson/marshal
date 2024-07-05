use std::any::{Any, type_name, TypeId};
use std::fmt::{Debug, Display, Formatter};

pub trait DerefRaw {
    type RawTarget: ?Sized;
    fn deref_raw(&self) -> *const Self::RawTarget;
}

pub trait RawAny: Any {
    fn raw_type_id(self: *const Self) -> TypeId;
    fn raw_type_name(self: *const Self) -> &'static str;
}

impl<T: Any> RawAny for T {
    fn raw_type_id(self: *const Self) -> TypeId {
        TypeId::of::<T>()
    }
    fn raw_type_name(self: *const Self) -> &'static str {
        type_name::<T>()
    }
}

impl dyn RawAny {
    pub fn downcast_check<T: 'static>(self: *const Self) -> Result<(), DowncastError<()>> {
        if self.raw_type_id() == TypeId::of::<T>() {
            Ok(())
        } else {
            Err(DowncastError {
                from: self.raw_type_name(),
                to: type_name::<T>(),
                inner: (),
            })
        }
    }
}

pub struct DowncastError<E> {
    from: &'static str,
    to: &'static str,
    inner: E,
}

impl<E> DowncastError<E> {
    pub fn map<E2>(self, e: impl FnOnce(E) -> E2) -> DowncastError<E2> {
        DowncastError {
            from: self.from,
            to: self.from,
            inner: e(self.inner),
        }
    }
}

impl<E> Debug for DowncastError<E> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DowncastError")
            .field("from", &self.from)
            .field("to", &self.to)
            .finish()
    }
}

impl<E> Display for DowncastError<E> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "tried to downcast from {} to {}", self.from, self.to)
    }
}

impl<E> std::error::Error for DowncastError<E> {}

pub trait DowncastRef<T: ?Sized> {
    fn downcast_ref(&self) -> Result<&T, DowncastError<()>>;
}
