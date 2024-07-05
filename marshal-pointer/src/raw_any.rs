use std::any::{Any, type_name, TypeId};
use std::fmt::{Display, Formatter};

pub trait AsFlatRef {
    type FlatRef: ?Sized;
    fn as_flat_ref(&self) -> &Self::FlatRef;
}

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
    pub fn downcast_check<T: 'static>(self: *const Self) -> Result<(), DowncastError> {
        if self.raw_type_id() == TypeId::of::<T>() {
            Ok(())
        } else {
            Err(DowncastError {
                from: self.raw_type_name(),
                to: type_name::<T>(),
            })
        }
    }
}

#[derive(Debug)]
pub struct DowncastError {
    from: &'static str,
    to: &'static str,
}

impl Display for DowncastError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "tried to downcast from {} to {}", self.from, self.to)
    }
}

impl std::error::Error for DowncastError {}

pub trait DowncastRef<T: ?Sized> {
    fn downcast_ref(&self) -> Result<&T, DowncastError>;
}