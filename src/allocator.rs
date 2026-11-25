//! Kernel heap: makes `Box`, `Vec`, `String` and friends work.
//!
//! The heap is a run of physical frames taken from the frame allocator.
//! The bootloader identity-maps the first 1 GiB, so those frames are usable
//! as-is with no extra page mapping. A linked-list allocator hands out pieces
//! of that region and takes them back on free.

use core::alloc::{GlobalAlloc, Layout};
use core::ptr::{self, NonNull};

use linked_list_allocator::Heap;
use spin::Mutex;
use x86_64::instructions::interrupts;

pub const HEAP_PAGES: u64 = 1024; // 4 MiB
pub const HEAP_SIZE: usize = (HEAP_PAGES * 4096) as usize;

struct KernelHeap(Mutex<Heap>);

#[global_allocator]
static HEAP: KernelHeap = KernelHeap(Mutex::new(Heap::empty()));

// Interrupts are disabled while the lock is held: if an interrupt handler
// tried to allocate while the interrupted code held the lock, it would spin
// forever on a lock that can never be released.
unsafe impl GlobalAlloc for KernelHeap {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        interrupts::without_interrupts(|| {
            self.0
                .lock()
                .allocate_first_fit(layout)
                .map_or(ptr::null_mut(), NonNull::as_ptr)
        })
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        interrupts::without_interrupts(|| unsafe {
            self.0
                .lock()
                .deallocate(NonNull::new_unchecked(ptr), layout)
        })
    }
}

/// Reserve the heap's frames and hand them to the allocator.
/// Returns the heap's start address, or `None` if RAM ran out.
pub fn init() -> Option<u64> {
    let start = crate::memory::alloc_pages(HEAP_PAGES)?;
    interrupts::without_interrupts(|| unsafe {
        HEAP.0.lock().init(start as *mut u8, HEAP_SIZE);
    });
    Some(start)
}

/// (used, free) bytes, for diagnostics.
pub fn stats() -> (usize, usize) {
    interrupts::without_interrupts(|| {
        let h = HEAP.0.lock();
        (h.used(), h.free())
    })
}
