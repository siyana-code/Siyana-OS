//! Runtime for apps running on Siyana: the `_start` entry point, the heap
//! and the panic handler. Linux apps get these from `std` instead.

use core::alloc::{GlobalAlloc, Layout};
use core::ptr::{self, NonNull};
use linked_list_allocator::LockedHeap;

use crate::backend;

unsafe extern "Rust" {
    /// Defined by `libsiyana::entry!` in the app.
    fn __siyana_main() -> i32;
}

/// Process entry point. The kernel jumps here with `rsp` pointing at the
/// top of the stack; realign it to 16 bytes as the Rust ABI expects.
#[unsafe(naked)]
#[unsafe(no_mangle)]
pub extern "C" fn _start() -> ! {
    core::arch::naked_asm!(
        "xor ebp, ebp",
        "and rsp, -16",
        "call {start}",
        "ud2",
        start = sym start_rust,
    )
}

extern "C" fn start_rust() -> ! {
    let code = unsafe { __siyana_main() };
    backend::exit(code)
}

#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    crate::eprintln!("panic: {info}");
    backend::exit(101)
}

// ------------------------------------------------------------------ heap

/// One region from `mem_map`, reserved on first use. The kernel backs
/// pages lazily, so reserving more than is used costs only address space.
const HEAP_SIZE: usize = 64 * 1024 * 1024;

struct UserHeap(LockedHeap);

#[global_allocator]
static HEAP: UserHeap = UserHeap(LockedHeap::empty());

unsafe impl GlobalAlloc for UserHeap {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let mut heap = self.0.lock();
        if heap.size() == 0 {
            match backend::mem_map(HEAP_SIZE) {
                Ok(base) => unsafe { heap.init(base, HEAP_SIZE) },
                Err(_) => return ptr::null_mut(),
            }
        }
        heap.allocate_first_fit(layout).map_or(ptr::null_mut(), NonNull::as_ptr)
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        unsafe { self.0.lock().deallocate(NonNull::new_unchecked(ptr), layout) }
    }
}
