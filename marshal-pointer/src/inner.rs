use std::alloc::Layout;
use std::ptr::NonNull;

use crate::raw_any::{DowncastError, RawAny};
use crate::raw_count::RawCount;

#[repr(C)]
pub struct Inner<C, T: ?Sized> {
    count: C,
    value: T,
}

impl<C: RawCount, T: ?Sized> Inner<C, T> {
    pub fn new(count: C, value: T) -> NonNull<Self>
    where
        T: Sized,
    {
        NonNull::new(Box::into_raw(Box::new(Inner { count, value }))).unwrap()
    }
    pub unsafe fn count_raw<'a>(self: *const Self) -> *const C {
        &(*self).count
    }
    pub unsafe fn into_raw(self: *const Self) -> *const T {
        &raw const (*self).value
    }
    pub unsafe fn from_raw(ptr: *const T) -> *const Self {
        let offset = Layout::new::<C>()
            .extend(Layout::for_value_raw(ptr))
            .unwrap()
            .1;
        ptr.byte_sub(offset) as *const Self
    }
}
impl<C: RawCount> Inner<C, dyn RawAny> {
    pub fn downcast_inner<T: 'static>(
        self: *const Self,
    ) -> Result<*const Inner<C, T>, DowncastError<*const Self>> {
        unsafe {
            self.into_raw()
                .downcast_check::<T>()
                .map_err(|e| e.map(|()| self))?;
            Ok(self as *const Inner<C, T>)
        }
    }
}
