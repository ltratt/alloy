#![allow(missing_docs)]
#![allow(unused_imports)]

#[cfg(not(test))]
#[cfg(not(no_global_oom_handling))]
use crate::boxed::Box;
#[cfg(test)]
#[cfg(not(no_global_oom_handling))]
use std::boxed::Box;

use core::{
    alloc::{AllocError, Allocator, GlobalAlloc, Layout},
    any::Any,
    borrow, cmp,
    cmp::Ordering,
    fmt,
    hash::{Hash, Hasher},
    marker::{FinalizerSafe, PhantomData, Unsize},
    mem::MaybeUninit,
    ops::{CoerceUnsized, Deref, DispatchFromDyn, Receiver},
    ptr,
    ptr::{drop_in_place, null_mut, NonNull},
};

#[cfg(profile_gc)]
use core::sync::atomic::{self, AtomicU64};

#[cfg(not(no_global_oom_handling))]
use core::gc::ReferenceFree;

// Fast-path for low alignment values
pub const MIN_ALIGN: usize = 8;

#[derive(Debug)]
pub struct GcAllocator;

unsafe impl GlobalAlloc for GcAllocator {
    #[inline]
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        unsafe { gc_malloc(layout) }
    }

    #[inline]
    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        unsafe { gc_free(ptr, layout) }
    }

    #[inline]
    unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
        unsafe { gc_realloc(ptr, layout, new_size) }
    }
}

#[inline]
unsafe fn gc_malloc(layout: Layout) -> *mut u8 {
    if layout.align() <= MIN_ALIGN && layout.align() <= layout.size() {
        unsafe { boehm::GC_malloc(layout.size()) as *mut u8 }
    } else {
        unsafe { boehm::GC_memalign(layout.align(), layout.size()) as *mut u8 }
    }
}

#[inline]
unsafe fn gc_realloc(ptr: *mut u8, old_layout: Layout, new_size: usize) -> *mut u8 {
    if old_layout.align() <= MIN_ALIGN && old_layout.align() <= new_size {
        unsafe { boehm::GC_realloc(ptr, new_size) as *mut u8 }
    } else {
        unsafe {
            let new_layout = Layout::from_size_align_unchecked(new_size, old_layout.align());

            let new_ptr = gc_malloc(new_layout);
            if !new_ptr.is_null() {
                let size = cmp::min(old_layout.size(), new_size);
                ptr::copy_nonoverlapping(ptr, new_ptr, size);
                gc_free(ptr, old_layout);
            }
            new_ptr
        }
    }
}

#[inline]
unsafe fn gc_free(ptr: *mut u8, _: Layout) {
    unsafe {
        boehm::GC_free(ptr);
    }
}

unsafe impl Allocator for GcAllocator {
    #[inline]
    fn allocate(&self, layout: Layout) -> Result<NonNull<[u8]>, AllocError> {
        match layout.size() {
            0 => Ok(NonNull::slice_from_raw_parts(layout.dangling(), 0)),
            size => unsafe {
                let ptr = gc_malloc(layout);
                let ptr = NonNull::new(ptr).ok_or(AllocError)?;
                Ok(NonNull::slice_from_raw_parts(ptr, size))
            },
        }
    }

    unsafe fn deallocate(&self, _: NonNull<u8>, _: Layout) {}
}

impl GcAllocator {
    pub fn force_gc() {
        unsafe { boehm::GC_gcollect() }
    }
}

pub fn init() {
    unsafe { boehm::GC_init() }
}

pub fn suppress_warnings() {
    unsafe { boehm::GC_set_warn_proc(&boehm::GC_ignore_warn_proc as *const _ as *mut u8) };
}

pub fn thread_registered() -> bool {
    unsafe { boehm::GC_thread_is_registered() != 0 }
}
