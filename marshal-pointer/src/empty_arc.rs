use std::{marker::Unsize, mem, ops::CoerceUnsized, sync, sync::Arc};
use std::any::TypeId;

use crate::arc_inner::ArcInner;
use crate::RawAny;

pub struct EmptyArc<T: ?Sized>(*mut ArcInner<T>);

impl<T: ?Sized> EmptyArc<T> {
    pub fn new() -> Self
    where
        T: Sized,
    {
        unsafe {
            let ptr = ArcInner::<T>::allocate_uninit();
            ptr.write_weak(1);
            ptr.write_strong(0);
            EmptyArc(ptr)
        }
    }
    pub fn downgrade(this: &Self) -> sync::Weak<T> {
        unsafe {
            this.0.increment_weak();
            this.0.into_weak()
        }
    }
    pub fn into_arc(this: Self, value: T) -> Arc<T>
    where
        T: Sized,
    {
        unsafe {
            this.0.write_inner(value);
            this.0.write_strong(1);
            let result = this.0.into_arc();
            mem::forget(this);
            result
        }
    }
}

impl EmptyArc<dyn RawAny> {
    pub fn downcast<T>(this: Self) -> Result<EmptyArc<T>, Self>
    where
        T: 'static,
    {
        unsafe {
            if this.0.inner_mut().raw_type_id() == TypeId::of::<T>() {
                let result = Ok(EmptyArc(this.0.cast()));
                mem::forget(this);
                result
            } else {
                Err(this)
            }
        }
    }
}

impl<T: ?Sized> Drop for EmptyArc<T> {
    fn drop(&mut self) {
        unsafe {
            self.0.into_weak();
        }
    }
}

impl<T: ?Sized + Unsize<U>, U: ?Sized> CoerceUnsized<EmptyArc<U>> for EmptyArc<T> {}

#[cfg(test)]
mod test {
    use std::mem::MaybeUninit;
    use std::sync::Arc;

    use crate::empty_arc::EmptyArc;

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
    fn test_uninit() {
        let _x = EmptyArc::<MaybeUninit<MustDrop>>::new();
    }

    #[test]
    fn test_uninit_arc() {
        let mut assert = AssertDropped::new();
        let x = EmptyArc::<MustDrop>::new();
        let x = EmptyArc::into_arc(x, assert.check());
        println!("{:?}", Arc::weak_count(&x));
        println!("{:?}", Arc::strong_count(&x));
    }

    #[test]
    fn test_uninit_weak() {
        let mut assert = AssertDropped::new();
        let x = EmptyArc::<MustDrop>::new();
        let w = EmptyArc::downgrade(&x);
        assert!(w.upgrade().is_none());
        EmptyArc::into_arc(x, assert.check());
    }
}
