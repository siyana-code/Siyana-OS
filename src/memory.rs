//! Physical memory discovery and allocation.
//!
//! Stage 2 queries the BIOS E820 memory map and leaves it at physical
//! 0x7E00 (20-byte entries, count at 0x7C20). We parse it, print the
//! usable regions, and run a simple bump frame allocator over RAM.

use core::sync::atomic::{AtomicU64, Ordering};

const E820_COUNT_ADDR: *const u16 = 0x7C20 as *const u16;
const E820_TABLE: *const u8 = 0x7E00 as *const u8;
const E820_ENTRY_SIZE: usize = 20;
const E820_TYPE_USABLE: u32 = 1;

const PAGE_SIZE: u64 = 4096;
// First 1 MiB is reserved (BIOS, VGA, etc.); start the heap there.
const HEAP_START: u64 = 0x10_0000;

static NEXT_FREE: AtomicU64 = AtomicU64::new(HEAP_START);

fn entry_base(i: usize) -> u64 {
    unsafe {
        let p = E820_TABLE.add(i * E820_ENTRY_SIZE);
        let lo = core::ptr::read_unaligned(p as *const u32) as u64;
        let hi = core::ptr::read_unaligned(p.add(4) as *const u32) as u64;
        lo | (hi << 32)
    }
}

fn entry_len(i: usize) -> u64 {
    unsafe {
        let p = E820_TABLE.add(i * E820_ENTRY_SIZE);
        let lo = core::ptr::read_unaligned(p.add(8) as *const u32) as u64;
        let hi = core::ptr::read_unaligned(p.add(12) as *const u32) as u64;
        lo | (hi << 32)
    }
}

fn entry_type(i: usize) -> u32 {
    unsafe { core::ptr::read_unaligned(E820_TABLE.add(i * E820_ENTRY_SIZE + 16) as *const u32) }
}

pub fn probe_and_print() {
    let count = unsafe { core::ptr::read_volatile(E820_COUNT_ADDR) } as usize;
    let mut usable_total: u64 = 0;

    let mut line = [0u8; 64];
    let prefix = b"E820 entries: ";
    line[..prefix.len()].copy_from_slice(prefix);
    line[prefix.len()] = b'0' + count as u8;
    if let Ok(s) = core::str::from_utf8(&line[..prefix.len() + 1]) {
        crate::fb::draw_text(10, 90, s, 255, 255, 255);
    }

    let hi = b"usable RAM (bytes): ";
    let mut total_ascii = [0u8; 32];
    let mut n = 0u64;
    for i in 0..count {
        if entry_type(i) == E820_TYPE_USABLE {
            n += entry_len(i);
        }
    }
    usable_total = n;
    // print total in MB for simplicity
    let mb = usable_total / (1024 * 1024);
    let mut buf = [0u8; 32];
    let pre = b"usable RAM ~ ";
    buf[..pre.len()].copy_from_slice(pre);
    let mut x = mb;
    let mut d = [0u8; 16];
    let mut l = 0;
    if x == 0 { d[0] = b'0'; l = 1; } else { while x > 0 { d[l] = b'0' + (x % 10) as u8; x /= 10; l += 1; } }
    for i in 0..l { buf[pre.len() + i] = d[l - 1 - i]; }
    let tail = b" MB";
    buf[pre.len() + l..pre.len() + l + tail.len()].copy_from_slice(tail);
    if let Ok(s) = core::str::from_utf8(&buf[..pre.len() + l + tail.len()]) {
        crate::fb::draw_text(10, 106, s, 0, 255, 255);
    }

    // Exercise the allocator: grab 8 pages (32 KiB) and report the start.
    if let Some(addr) = alloc_pages(8) {
        let msg = b"alloc 8 pages @ 0x";
        let mut h = [0u8; 64];
        h[..msg.len()].copy_from_slice(msg);
        let mut v = addr;
        let hex_chars = b"0123456789ABCDEF";
        for i in (0..8).rev() {
            h[msg.len() + (7 - i)] = hex_chars[((v >> (i * 4)) & 0xF) as usize];
        }
        if let Ok(s) = core::str::from_utf8(&h[..msg.len() + 8]) {
            crate::fb::draw_text(10, 122, s, 255, 255, 0);
        }
    }
}

/// Allocate `n` contiguous 4 KiB pages. Returns the start address.
pub fn alloc_pages(n: u64) -> Option<u64> {
    let size = n * PAGE_SIZE;
    let addr = NEXT_FREE.fetch_add(size, Ordering::SeqCst);
    // crude bound: no real limit yet (fine for a demo; a real kernel
    // checks against the E820 map and a free list).
    if addr + size < 0x800_0000 {
        Some(addr)
    } else {
        None
    }
}
