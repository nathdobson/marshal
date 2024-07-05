use std::{
    any::Any,
    marker::Unsize,
    mem,
    ops::{CoerceUnsized, Deref, DerefMut},
    rc,
};
use std::rc::Rc;

use crate::inner::RcInner;

pub struct UniqueRc<T: ?Sized>(*mut RcInner<T>);

impl<T: ?Sized> UniqueRc<T> {
    pub fn new(value: T) -> Self
    where
        T: Sized,
    {
        unsafe {
            let ptr = RcInner::<T>::allocate_uninit();
            ptr.write_weak(1);
            ptr.write_strong(0);
            ptr.write_inner(value);
            UniqueRc(ptr)
        }
    }
    pub fn downgrade(this: &Self) -> rc::Weak<T> {
        unsafe {
            this.0.increment_weak();
            this.0.into_weak()
        }
    }
    pub fn into_arc(this: Self) -> Rc<T> {
        unsafe {
            this.0.increment_strong();
            let result = this.0.into_strong();
            mem::forget(this);
            result
        }
    }
}

impl UniqueRc<dyn 'static + Any> {
    pub fn downcast<T>(this: Self) -> Result<UniqueRc<T>, Self>
    where
        T: Any + Send + Sync,
    {
        if (*this).is::<T>() {
            let result = Ok(UniqueRc(this.0.cast()));
            mem::forget(this);
            result
        } else {
            Err(this)
        }
    }
}

impl<T: ?Sized> Deref for UniqueRc<T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        unsafe { self.0.inner_deref() }
    }
}

impl<T: ?Sized> DerefMut for UniqueRc<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { self.0.inner_deref_mut() }
    }
}

impl<T: ?Sized> Drop for UniqueRc<T> {
    fn drop(&mut self) {
        unsafe {
            self.0.drop_inner();
            self.0.into_weak();
        }
    }
}

impl<T: ?Sized + Unsize<U>, U: ?Sized> CoerceUnsized<UniqueRc<U>> for UniqueRc<T> {}

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
