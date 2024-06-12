use std::{
    alloc::{Allocator, Global, handle_alloc_error, Layout},
    any::Any,
    cell::UnsafeCell,
    marker::Unsize,
    mem,
    mem::MaybeUninit,
    ops::{CoerceUnsized, Deref, DerefMut},
    ptr::NonNull,
    sync::{
        Arc,
        atomic::{AtomicUsize, Ordering}, Weak,
    },
};

#[repr(C)]
struct ArcInner<T: ?Sized> {
    strong: AtomicUsize,
    weak: AtomicUsize,
    inner: UnsafeCell<T>,
}

pub struct UniqueArc<T: ?Sized>(NonNull<ArcInner<T>>);

const MAX_REFCOUNT: usize = (isize::MAX) as usize;

impl<T: ?Sized> UniqueArc<T> {
    pub fn new(value: T) -> Self
        where
            T: Sized,
    {
        UniqueArc(
            NonNull::new(Box::into_raw(Box::new(ArcInner {
                strong: AtomicUsize::new(0),
                weak: AtomicUsize::new(1),
                inner: UnsafeCell::new(value),
            })))
                .unwrap(),
        )
    }
    pub fn downgrade(this: &Self) -> Weak<T> {
        unsafe {
            let count = this.0.as_ref().weak.fetch_add(1, Ordering::Relaxed);
            assert!(count <= MAX_REFCOUNT);
            Weak::from_raw(this.0.as_ref().inner.get())
        }
    }
    pub fn into_arc(self) -> Arc<T> {
        unsafe {
            let raw: *mut T = self.0.as_ref().inner.get();
            let count = self.0.as_ref().strong.fetch_add(1, Ordering::Release);
            assert!(count <= MAX_REFCOUNT);
            mem::forget(self);
            Arc::from_raw(raw)
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
        unsafe { &*self.0.as_ref().inner.get() }
    }
}

impl<T: ?Sized> DerefMut for UniqueArc<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut *self.0.as_ref().inner.get() }
    }
}

impl<T: ?Sized> Drop for UniqueArc<T> {
    fn drop(&mut self) {
        unsafe {
            self.0.as_ref().inner.get().drop_in_place();
            mem::drop(Weak::from_raw(self.0.as_ref().inner.get()));
        }
    }
}

impl<T> UniqueArc<MaybeUninit<T>> {
    pub fn new_uninit() -> Self {
        unsafe {
            let layout = Layout::new::<ArcInner<MaybeUninit<T>>>();
            let mut ptr = Global
                .allocate(layout)
                .unwrap_or_else(|_| handle_alloc_error(layout))
                .cast::<ArcInner<MaybeUninit<T>>>();
            *ptr.as_mut().strong.get_mut() = 0;
            *ptr.as_mut().weak.get_mut() = 1;
            UniqueArc(ptr)
        }
    }
    pub fn init(mut self, value: T) -> Arc<T> {
        unsafe {
            ((&raw mut (*self.0.as_ptr()).inner) as *mut T).write(value);
            mem::transmute::<Arc<MaybeUninit<T>>, Arc<T>>(self.into_arc())
        }
    }
    pub fn downgrade_uninit(this: &Self) -> Weak<T> {
        unsafe { mem::transmute::<Weak<MaybeUninit<T>>, Weak<T>>(Self::downgrade(this)) }
    }
}

impl<T: ?Sized + Unsize<U>, U: ?Sized> CoerceUnsized<UniqueArc<U>> for UniqueArc<T> {}

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
    let _x = x.into_arc();
}

#[test]
fn test_with_weak() {
    let mut assert = AssertDropped::new();
    let x = UniqueArc::new(assert.check());
    let w = UniqueArc::downgrade(&x);
    let _x = x.into_arc();
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
    x.init(assert.check());
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
    static mut ASSERT: AssertDropped = AssertDropped::new();
    let x = UniqueArc::<MaybeUninit<MustDrop>>::new_uninit();
    let x: UniqueArc<dyn 'static + Sync + Send + Any> = x;
    UniqueArc::downcast_downgrade_uninit::<MustDrop>(&x).unwrap();
}
