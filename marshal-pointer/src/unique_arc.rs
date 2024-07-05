use std::{
    any::Any,
    marker::Unsize,
    mem,
    ops::{CoerceUnsized, Deref, DerefMut},
    sync::{Arc, Weak},
};

use crate::inner::ArcInner;

pub struct UniqueArc<T: ?Sized>(*mut ArcInner<T>);

impl<T: ?Sized> UniqueArc<T> {
    pub fn new(value: T) -> Self
    where
        T: Sized,
    {
        unsafe {
            let ptr = ArcInner::<T>::allocate_uninit();
            ptr.write_weak(1);
            ptr.write_strong(0);
            ptr.write_inner(value);
            UniqueArc(ptr)
        }
    }
    pub fn downgrade(this: &Self) -> Weak<T> {
        unsafe {
            this.0.increment_weak();
            this.0.into_weak()
        }
    }
    pub fn into_arc(this: Self) -> Arc<T> {
        unsafe {
            this.0.increment_strong();
            let result = this.0.into_strong();
            mem::forget(this);
            result
        }
    }
}

impl UniqueArc<dyn 'static + Any> {
    pub fn downcast<T>(this: Self) -> Result<UniqueArc<T>, Self>
    where
        T: Any + Send + Sync,
    {
        if (*this).is::<T>() {
            let result = Ok(UniqueArc(this.0.cast()));
            mem::forget(this);
            result
        } else {
            Err(this)
        }
    }
}

impl<T: ?Sized> Deref for UniqueArc<T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        unsafe { self.0.inner_deref() }
    }
}

impl<T: ?Sized> DerefMut for UniqueArc<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { self.0.inner_deref_mut() }
    }
}

impl<T: ?Sized> Drop for UniqueArc<T> {
    fn drop(&mut self) {
        unsafe {
            self.0.drop_inner();
            self.0.into_weak();
        }
    }
}

impl<T: ?Sized + Unsize<U>, U: ?Sized> CoerceUnsized<UniqueArc<U>> for UniqueArc<T> {}

#[cfg(test)]
mod test {
    use std::mem;

    use crate::unique_arc::UniqueArc;

    struct AssertDropped {
        dropped: bool,
    }

    impl AssertDropped {
        pub const fn new() -> Self {
            AssertDropped { dropped: false }
        }
        pub fn check(&mut self) -> MustDrop {
            MustDrop(self)
        }
    }

    struct MustDrop<'a>(&'a mut AssertDropped);

    impl<'a> Drop for MustDrop<'a> {
        fn drop(&mut self) {
            assert!(!self.0.dropped);
            self.0.dropped = true;
        }
    }

    impl Drop for AssertDropped {
        fn drop(&mut self) {
            assert!(self.dropped);
        }
    }

    #[test]
    fn test_without_arc() {
        let mut assert = AssertDropped::new();
        let _x = UniqueArc::new(assert.check());
    }

    #[test]
    fn test_with_arc() {
        let mut assert = AssertDropped::new();
        let x = UniqueArc::new(assert.check());
        let _x = UniqueArc::into_arc(x);
    }

    #[test]
    fn test_with_weak() {
        let mut assert = AssertDropped::new();
        let x = UniqueArc::new(assert.check());
        let w = UniqueArc::downgrade(&x);
        let _x = UniqueArc::into_arc(x);
        mem::drop(w);
    }
}
