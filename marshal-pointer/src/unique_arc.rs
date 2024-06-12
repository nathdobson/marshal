use std::{
    any::Any,
    marker::Unsize,
    mem,
    mem::MaybeUninit,
    ops::{CoerceUnsized, Deref, DerefMut},
    sync::{Arc, Weak},
};

use crate::arc_inner::ArcInner;

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
            let result = this.0.into_arc();
            mem::forget(this);
            result
        }
    }
}

impl UniqueArc<dyn 'static + Sync + Send + Any> {
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
    pub fn downcast_downgrade<T: 'static>(this: &Self) -> Option<Weak<T>> {
        if (**this).is::<T>() {
            unsafe { Some(Weak::from_raw(Self::downgrade(this).into_raw() as *const T)) }
        } else {
            None
        }
    }
    pub fn downcast_downgrade_uninit<T: 'static>(this: &Self) -> Option<Weak<T>> {
        if (**this).is::<MaybeUninit<T>>() {
            unsafe { Some(Weak::from_raw(Self::downgrade(this).into_raw() as *const T)) }
        } else {
            None
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

impl<T> UniqueArc<MaybeUninit<T>> {
    pub fn new_uninit() -> Self {
        unsafe {
            let ptr = ArcInner::allocate_uninit();
            ptr.write_strong(0);
            ptr.write_weak(1);
            UniqueArc(ptr)
        }
    }
    pub fn init(self, value: T) -> Arc<T> {
        unsafe {
            self.0.write_inner(MaybeUninit::new(value));
            self.0.increment_strong();
            let result = (self.0 as *mut ArcInner<T>).into_arc();
            mem::forget(self);
            result
        }
    }
    pub fn downgrade_uninit(this: &Self) -> Weak<T> {
        unsafe { mem::transmute::<Weak<MaybeUninit<T>>, Weak<T>>(Self::downgrade(this)) }
    }
}

impl<T: ?Sized + Unsize<U>, U: ?Sized> CoerceUnsized<UniqueArc<U>> for UniqueArc<T> {}

#[cfg(test)]
mod test {
    use std::any::Any;
    use std::mem;
    use std::mem::MaybeUninit;
    use std::sync::Arc;
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

    #[test]
    fn test_uninit() {
        let _x = UniqueArc::<MaybeUninit<MustDrop>>::new_uninit();
    }

    #[test]
    fn test_uninit_arc() {
        let mut assert = AssertDropped::new();
        let x = UniqueArc::<MaybeUninit<MustDrop>>::new_uninit();
        let x = x.init(assert.check());
        println!("{:?}", Arc::weak_count(&x));
        println!("{:?}", Arc::strong_count(&x));
    }

    #[test]
    fn test_uninit_weak() {
        let mut assert = AssertDropped::new();
        let x = UniqueArc::<MaybeUninit<MustDrop>>::new_uninit();
        let w = UniqueArc::downgrade_uninit(&x);
        assert!(w.upgrade().is_none());
        x.init(assert.check());
    }

    #[test]
    fn test_downcast_downgrade_uninit() {
        let x = UniqueArc::<MaybeUninit<MustDrop>>::new_uninit();
        let x: UniqueArc<dyn 'static + Sync + Send + Any> = x;
        UniqueArc::downcast_downgrade_uninit::<MustDrop>(&x).unwrap();
    }
}
