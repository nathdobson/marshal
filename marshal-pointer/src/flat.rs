use std::any::Any;
use std::borrow::Borrow;
use std::fmt::{Debug, Display, Formatter};
use std::hash::Hash;
use std::marker::Unsize;
use std::ops::{CoerceUnsized, Deref, DispatchFromDyn};
use std::rc::Rc;
use std::sync::Arc;

use crate::{ArcWeak, AsFlatRef, DerefRaw, RcWeak};
use crate::strong_ref::{ArcRef, RcRef};
use crate::weak_ref::{ArcWeakRef, RcWeakRef};

macro_rules! rcf {
    ($strong_flat:ident, $weak_flat:ident, $strong_ref:ident,$weak_ref:ident, $strong:ident, $weak:ident) => {
        #[derive(Eq, PartialEq, Hash, Default, Ord, PartialOrd)]
        pub struct $strong_flat<T: ?Sized>($strong<T>);
        pub struct $weak_flat<T: ?Sized>($weak<T>);

        impl<T> From<T> for $strong_flat<T> {
            fn from(value: T) -> Self {
                $strong_flat($strong::new(value))
            }
        }

        impl<T: ?Sized> From<$strong<T>> for $strong_flat<T> {
            fn from(value: $strong<T>) -> Self {
                $strong_flat(value)
            }
        }

        impl<T: ?Sized> From<$weak<T>> for $weak_flat<T> {
            fn from(value: $weak<T>) -> Self {
                $weak_flat(value)
            }
        }

        impl<T: ?Sized> From<$strong_flat<T>> for $strong<T> {
            fn from(value: $strong_flat<T>) -> Self {
                value.0
            }
        }

        impl<T: ?Sized> From<$weak_flat<T>> for $weak<T> {
            fn from(value: $weak_flat<T>) -> Self {
                value.0
            }
        }

        impl<T: ?Sized> Deref for $strong_flat<T> {
            type Target = $strong_ref<T>;
            fn deref(&self) -> &Self::Target {
                self.0.as_flat_ref()
            }
        }

        impl<T: ?Sized> AsFlatRef for $strong_flat<T> {
            type FlatRef = $strong_ref<T>;
            fn as_flat_ref(&self) -> &Self::FlatRef {
                self.0.as_flat_ref()
            }
        }

        impl<T: ?Sized> AsFlatRef for $weak_flat<T> {
            type FlatRef = $weak_ref<T>;
            fn as_flat_ref(&self) -> &Self::FlatRef {
                self.0.as_flat_ref()
            }
        }

        impl<T: ?Sized> DerefRaw for $strong_flat<T> {
            type RawTarget = T;
            fn deref_raw(&self) -> *const Self::RawTarget {
                self.0.deref_raw()
            }
        }

        impl<T: ?Sized> DerefRaw for $weak_flat<T> {
            type RawTarget = T;
            fn deref_raw(&self) -> *const Self::RawTarget {
                self.0.deref_raw()
            }
        }

        impl<T: ?Sized> $strong_flat<T> {
            pub fn new(value: T) -> $strong_flat<T>
            where
                T: Sized,
            {
                $strong_flat($strong::new(value))
            }
            pub fn downgrade(this: &Self) -> $weak_flat<T> {
                $weak_flat($strong::downgrade(&this.0))
            }
            pub fn into_raw(this: Self) -> *const T {
                $strong::into_raw(this.0)
            }
            pub unsafe fn from_raw(ptr: *const T) -> Self {
                $strong_flat($strong::from_raw(ptr))
            }
        }

        impl<T: ?Sized> $weak_flat<T> {
            pub fn into_raw(this: Self) -> *const T {
                $weak::into_raw(this.0)
            }
            pub unsafe fn from_raw(ptr: *const T) -> Self {
                $weak_flat($weak::from_raw(ptr))
            }
            pub fn upgrade(&self) -> Option<$strong_flat<T>> {
                Some($strong_flat(self.0.upgrade()?))
            }
        }

        impl<T: ?Sized> AsRef<T> for $strong_flat<T> {
            fn as_ref(&self) -> &T {
                self.0.as_ref()
            }
        }

        impl<T: ?Sized> AsRef<$strong_ref<T>> for $strong_flat<T> {
            fn as_ref(&self) -> &$strong_ref<T> {
                self.0.as_flat_ref()
            }
        }

        impl<T: ?Sized> Borrow<T> for $strong_flat<T> {
            fn borrow(&self) -> &T {
                self.0.borrow()
            }
        }

        impl<T: ?Sized> Borrow<$strong_ref<T>> for $strong_flat<T> {
            fn borrow(&self) -> &$strong_ref<T> {
                self.0.as_flat_ref()
            }
        }

        impl<T: ?Sized> Clone for $strong_flat<T> {
            fn clone(&self) -> Self {
                $strong_flat(self.0.clone())
            }
        }

        impl<T: ?Sized> Clone for $weak_flat<T> {
            fn clone(&self) -> Self {
                $weak_flat(self.0.clone())
            }
        }

        impl<T, U> CoerceUnsized<$strong_flat<U>> for $strong_flat<T>
        where
            T: ?Sized + Unsize<U>,
            U: ?Sized,
        {
        }

        impl<T, U> CoerceUnsized<$weak_flat<U>> for $weak_flat<T>
        where
            T: ?Sized + Unsize<U>,
            U: ?Sized,
        {
        }

        impl<T, U> DispatchFromDyn<$strong_flat<U>> for $strong_flat<T>
        where
            T: ?Sized + Unsize<U>,
            U: ?Sized,
        {
        }

        impl<T: ?Sized + Display> Display for $strong_flat<T> {
            fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
                self.0.fmt(f)
            }
        }
        impl<T: ?Sized + Debug> Debug for $strong_flat<T> {
            fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
                self.0.fmt(f)
            }
        }
        impl<T: ?Sized + Display> Display for $weak_flat<T> {
            fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
                self.0.fmt(f)
            }
        }
        impl<T: ?Sized + Debug> Debug for $weak_flat<T> {
            fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
                self.0.fmt(f)
            }
        }

        #[cfg(feature = "weak-table")]
        impl<T: ?Sized> weak_table::traits::WeakElement for $weak_flat<T> {
            type Strong = $strong_flat<T>;
            fn new(view: &Self::Strong) -> Self {
                $strong_flat::downgrade(view)
            }
            fn view(&self) -> Option<Self::Strong> {
                self.upgrade()
            }
        }

        #[cfg(feature = "weak-table")]
        impl<T: ?Sized + Hash + Eq> weak_table::traits::WeakKey for $weak_flat<T> {
            type Key = T;

            fn with_key<F, R>(view: &Self::Strong, f: F) -> R
            where
                F: FnOnce(&Self::Key) -> R,
            {
                f(view)
            }
        }
    };
}

rcf!(Rcf, RcfWeak, RcRef, RcWeakRef, Rc, RcWeak);
rcf!(Arcf, ArcfWeak, ArcRef, ArcWeakRef, Arc, ArcWeak);

impl Arcf<dyn Sync + Send + Any> {
    pub fn downcast<T: Sync + Send + Any>(self) -> Result<Arcf<T>, Self> {
        self.0.downcast().map(Arcf).map_err(Arcf)
    }
}

impl Rcf<dyn Any> {
    pub fn downcast<T: Any>(self) -> Result<Rcf<T>, Self> {
        self.0.downcast().map(Rcf).map_err(Rcf)
    }
}
