//! Bump allocator for RINO.
//!
//! The simplest possible heap allocator:
//! - Maintains a pointer (`next`) into a heap region
//! - Each allocation advances `next` by the requested size (+ alignment)
//! - Deallocation is a no-op (memory is never freed)
//!
//! This is suitable for early boot, one-shot allocations, or systems
//! that never deallocate. A real OS would replace this with a more
//! sophisticated allocator (linked list, buddy, slab) later.

use core::alloc::{GlobalAlloc, Layout};
use core::cell::UnsafeCell;
use core::ptr;
use core::sync::atomic::{AtomicUsize, Ordering};

/// A bump allocator that hands out memory from a fixed-size region.
///
/// Thread safety: uses an `AtomicUsize` for the bump pointer.
/// This is correct for single-core use (which we are right now).
/// For multi-core, you'd need a spin lock or per-core arenas.
pub struct BumpAllocator {
    heap_start: UnsafeCell<usize>,
    heap_end: UnsafeCell<usize>,
    next: AtomicUsize,
}

// Required to use as #[global_allocator] — we promise to handle
// synchronization ourselves (via the atomic).
unsafe impl Sync for BumpAllocator {}

impl BumpAllocator {
    /// Create a new uninitialized bump allocator.
    /// Must call `init()` before any allocations.
    pub const fn uninit() -> Self {
        Self {
            heap_start: UnsafeCell::new(0),
            heap_end: UnsafeCell::new(0),
            next: AtomicUsize::new(0),
        }
    }

    /// Initialize the allocator with a heap memory region.
    ///
    /// # Safety
    /// - `heap_start` must point to `heap_size` bytes of valid, unused memory
    /// - This must be called exactly once, before any allocations
    pub unsafe fn init(&self, heap_start: *mut u8, heap_size: usize) {
        unsafe {
            *self.heap_start.get() = heap_start as usize;
            *self.heap_end.get() = heap_start as usize + heap_size;
        }
        self.next.store(heap_start as usize, Ordering::Release);
    }
}

unsafe impl GlobalAlloc for BumpAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        loop {
            let current = self.next.load(Ordering::Relaxed);

            // Align up: round `current` up to the nearest multiple of `layout.align()`
            // Example: if current = 0x1003 and align = 4, aligned = 0x1004
            let aligned = (current + layout.align() - 1) & !(layout.align() - 1);

            let new_next = aligned + layout.size();

            let heap_end = unsafe { *self.heap_end.get() };
            if new_next > heap_end {
                // Out of memory
                return ptr::null_mut();
            }

            // Try to advance the bump pointer atomically.
            // If another core allocated between our load and this store,
            // compare_exchange fails and we retry the loop.
            if self
                .next
                .compare_exchange(current, new_next, Ordering::Relaxed, Ordering::Relaxed)
                .is_ok()
            {
                return aligned as *mut u8;
            }
        }
    }

    unsafe fn dealloc(&self, _ptr: *mut u8, _layout: Layout) {
        // Bump allocator never frees. This is intentional.
        // Memory is reclaimed only when the entire heap is reset.
    }
}

/// The global allocator instance.
/// This must be initialized via `init()` before any heap allocations.
#[global_allocator]
pub static ALLOCATOR: BumpAllocator = BumpAllocator::uninit();

/// Initialize the heap allocator with memory from the platform.
///
/// Call this once during boot, after the platform is initialized.
pub fn init(heap_start: *mut u8, heap_size: usize) {
    unsafe { ALLOCATOR.init(heap_start, heap_size) };
}
