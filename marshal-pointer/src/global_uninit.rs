use std::alloc::{handle_alloc_error, Allocator, Global, Layout};
use std::intrinsics::likely;
use std::mem::{align_of_val_raw, size_of_val_raw, MaybeUninit};

use safe_once::sync::OnceLock;

#[allow(dead_code)]
#[repr(align(1024))]
struct SmallMemory(MaybeUninit<[u8; 1024]>);
static SMALL_MEMORY: SmallMemory = SmallMemory(MaybeUninit::uninit());

#[inline]
pub unsafe fn global_uninit_for_ptr<T: ?Sized>(ptr: *const T) -> *const T {
    global_uninit(Layout::from_size_align(size_of_val_raw(ptr), align_of_val_raw(ptr)).unwrap())
        .with_metadata_of(ptr)
}

#[inline]
pub unsafe fn global_uninit(layout: Layout) -> *const () {
    let max = layout.size().max(layout.align());
    if likely(max <= size_of::<SmallMemory>()) {
        &SMALL_MEMORY as *const SmallMemory as *const ()
    } else {
        global_uninit_slow(max)
    }
}

struct Pointer(*const ());
unsafe impl Send for Pointer {}
unsafe impl Sync for Pointer {}
const ONCE_LOCK: OnceLock<Pointer> = OnceLock::new();
static LARGE_MEMORY: [OnceLock<Pointer>; 64] = [ONCE_LOCK; 64];
unsafe fn global_uninit_slow(size_align: usize) -> *const () {
    let index = (size_align - 1).ilog2() as usize;
    let size = 1 << (index + 1);
    let result = LARGE_MEMORY[index].get_or_init(|| {
        assert!(size >= size_align);
        let layout = Layout::from_size_align(size, size).unwrap();
        let result = Global
            .allocate(layout)
            .unwrap_or_else(|_| handle_alloc_error(layout))
            .as_ptr() as *const ();
        assert_eq!(result as usize % size_align, 0);
        Pointer(result)
    });
    result.0
}
