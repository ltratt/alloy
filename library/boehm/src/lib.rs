#![no_std]

#[repr(C)]
#[derive(Default)]
pub struct ProfileStats {
    /// Heap size in bytes (including area unmapped to OS).
    pub heapsize_full: usize,
    /// Total bytes contained in free and unmapped blocks.
    pub free_bytes_full: usize,
    /// Amount of memory unmapped to OS.
    pub unmapped_bytes: usize,
    /// Number of bytes allocated since the recent collection.
    pub bytes_allocd_since_gc: usize,
    /// Number of bytes allocated before the recent collection.
    /// The value may wrap.
    pub allocd_bytes_before_gc: usize,
    /// Number of bytes not considered candidates for garbage collection.
    pub non_gc_bytes: usize,
    /// Garbage collection cycle number.
    /// The value may wrap.
    pub gc_no: usize,
    /// Number of marker threads (excluding the initiating one).
    pub markers_m1: usize,
    /// Approximate number of reclaimed bytes after recent collection.
    pub bytes_reclaimed_since_gc: usize,
    /// Approximate number of bytes reclaimed before the recent collection.
    /// The value may wrap.
    pub reclaimed_bytes_before_gc: usize,
    /// Number of bytes freed explicitly since the recent GC.
    pub expl_freed_bytes_since_gc: usize,
}

#[link(name = "gc")]
extern "C" {
    pub fn GC_malloc(nbytes: usize) -> *mut u8;

    pub fn GC_realloc(old: *mut u8, new_size: usize) -> *mut u8;

    pub fn GC_free(dead: *mut u8);

    pub fn GC_register_finalizer(
        ptr: *mut u8,
        finalizer: Option<unsafe extern "C" fn(*mut u8, *mut u8)>,
        client_data: *mut u8,
        old_finalizer: *mut extern "C" fn(*mut u8, *mut u8),
        old_client_data: *mut *mut u8,
    );

    pub fn GC_register_finalizer_no_order(
        ptr: *mut u8,
        finalizer: Option<unsafe extern "C" fn(*mut u8, *mut u8)>,
        client_data: *mut u8,
        old_finalizer: *mut extern "C" fn(*mut u8, *mut u8),
        old_client_data: *mut *mut u8,
    );

    pub fn GC_gcollect();

    pub fn GC_thread_is_registered() -> u32;

    pub fn GC_register_my_thread(stack_base: *mut u8) -> i32;

    pub fn GC_unregister_my_thread() -> i32;

    pub fn GC_init();

    pub fn GC_set_warn_proc(level: *mut u8);

    pub fn GC_ignore_warn_proc(proc: *mut u8, word: usize);
}
