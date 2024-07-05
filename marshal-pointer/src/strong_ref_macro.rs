use std::any::TypeId;
use std::marker::PhantomData;
use std::ops::Deref;
use std::rc::Rc;
use std::sync::Arc;

use crate::{ArcWeak, AsFlatRef, DerefRaw, DowncastRef, RawAny, RcWeak};
use crate::flat::{Arcf, ArcfWeak, Rcf, RcfWeak};

macro_rules! rc_ref {
    (
        $strong_ref:ident,
        $strong:ident,
        $weak:ident,
        $strong_flat:ident,
        $weak_flat:ident,
        $strong_method:ident,
        $strong_flat_method:ident,
        $weak_method:ident,
        $weak_flat_method:ident
    ) => {
        #[repr(transparent)]
        pub struct $strong_ref<T: ?Sized> {
            phantom: PhantomData<*const ()>,
            inner: T,
        }

        impl<T: ?Sized> AsFlatRef for $strong<T> {
            type FlatRef = $strong_ref<T>;
            fn as_flat_ref(&self) -> &Self::FlatRef {
                unsafe { &*((&**self) as *const T as *const $strong_ref<T>) }
            }
        }

        impl<T: ?Sized> $strong_ref<T> {
            pub fn $strong_method(&self) -> $strong<T> {
                unsafe {
                    <$strong<T>>::increment_strong_count(&**self);
                    <$strong<T>>::from_raw(self as *const $strong_ref<T> as *const T)
                }
            }
            pub fn $strong_flat_method(&self) -> $strong_flat<T> {
                self.$strong_method().into()
            }
            pub fn $weak_method(&self) -> $weak<T> {
                <$strong<T>>::downgrade(&self.$strong_method())
            }
            pub fn $weak_flat_method(&self) -> $weak_flat<T> {
                self.$weak_method().into()
            }
        }

        impl<T: ?Sized> Deref for $strong_ref<T> {
            type Target = T;
            fn deref(&self) -> &Self::Target {
                &self.inner
            }
        }

        impl<T: ?Sized> DerefRaw for $strong<T> {
            type RawTarget = T;
            fn deref_raw(&self) -> *const Self::RawTarget {
                &**self
            }
        }

        impl<T: ?Sized> DerefRaw for $strong_ref<T> {
            type RawTarget = T;
            fn deref_raw(&self) -> *const Self::RawTarget {
                &**self
            }
        }

        impl<T: 'static> DowncastRef<$strong_ref<T>> for $strong_ref<dyn RawAny> {
            fn downcast_ref(&self) -> Option<&$strong_ref<T>> {
                unsafe {
                    if self.deref_raw().raw_type_id() == TypeId::of::<T>() {
                        Some(&*(self as *const $strong_ref<dyn RawAny> as *const $strong_ref<T>))
                    } else {
                        None
                    }
                }
            }
        }

        impl<T> AsRef<$strong_ref<T>> for $strong<T> {
            fn as_ref(&self) -> &$strong_ref<T> {
                self.as_flat_ref()
            }
        }
    };
}

rc_ref!(RcRef, Rc, RcWeak, Rcf, RcfWeak, rc, rcf, rc_weak, rcf_weak);
rc_ref!(ArcRef, Arc, ArcWeak, Arcf, ArcfWeak, arc, arcf, arc_weak, arcf_weak);
