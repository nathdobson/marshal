use std::rc::Rc;
use std::{
    any::Any,
    marker::Unsize,
    mem,
    mem::MaybeUninit,
    ops::{CoerceUnsized, Deref, DerefMut},
    rc,
    sync::Arc,
};

use crate::arc_inner::ArcInner;
use crate::rc_inner::RcInner;

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
    pub fn into_rc(this: Self) -> Rc<T> {
        unsafe {
            this.0.write_strong(1);
            let result = this.0.into_rc();
            mem::forget(this);
            result
        }
    }
}

impl UniqueRc<dyn 'static + Any> {
    pub fn downcast<T>(this: Self) -> Result<UniqueRc<T>, Self>
    where
        T: Any,
    {
        if (*this).is::<T>() {
            let result = Ok(UniqueRc(this.0.cast()));
            mem::forget(this);
            result
        } else {
            Err(this)
        }
    }
    pub fn downcast_downgrade<T: 'static>(this: &Self) -> Option<rc::Weak<T>> {
        if (**this).is::<T>() {
            unsafe {
                Some(rc::Weak::from_raw(
                    Self::downgrade(this).into_raw() as *const T
                ))
            }
        } else {
            None
        }
    }
    pub fn downcast_downgrade_uninit<T: 'static>(this: &Self) -> Option<rc::Weak<T>> {
        if (**this).is::<MaybeUninit<T>>() {
            unsafe {
                Some(rc::Weak::from_raw(
                    Self::downgrade(this).into_raw() as *const T
                ))
            }
        } else {
            None
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

impl<T> UniqueRc<MaybeUninit<T>> {
    pub fn new_uninit() -> Self {
        unsafe {
            let ptr = RcInner::allocate_uninit();
            ptr.write_strong(0);
            ptr.write_weak(1);
            UniqueRc(ptr)
        }
    }
    pub fn init(self, value: T) -> Rc<T> {
        unsafe {
            self.0.write_inner(MaybeUninit::new(value));
            self.0.write_strong(1);
            let result = (self.0 as *mut RcInner<T>).into_rc();
            mem::forget(self);
            result
        }
    }
    pub fn downgrade_uninit(this: &Self) -> rc::Weak<T> {
        unsafe { mem::transmute::<rc::Weak<MaybeUninit<T>>, rc::Weak<T>>(Self::downgrade(this)) }
    }
}

impl<T: ?Sized + Unsize<U>, U: ?Sized> CoerceUnsized<UniqueRc<U>> for UniqueRc<T> {}

#[cfg(test)]
mod test {
    use crate::unique_arc::UniqueArc;
    use std::any::Any;
    use std::mem;
    use std::mem::MaybeUninit;
    use std::sync::Arc;

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
