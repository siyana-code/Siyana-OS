#![no_std]
#![no_main]
#![feature(abi_x86_interrupt)]

extern crate alloc;

mod allocator;
mod interrupts;
mod fb;
mod memory;
mod font;
mod keyboard;

use core::panic::PanicInfo;

#[unsafe(no_mangle)]
#[unsafe(link_section = ".text.start")]
extern "C" fn _start() -> ! {
    // .bss is not stored in kernel.bin, so the loader never writes it.
    // Zero it before any static is touched.
    unsafe extern "C" {
        static mut __bss_start: u8;
        static mut __bss_end: u8;
    }
    unsafe {
        let start = &raw mut __bss_start;
        let end = &raw mut __bss_end;
        core::ptr::write_bytes(start, 0, end as usize - start as usize);
    }

    let vga = 0xB8140 as *mut u8;
    let message = b"Siyana kernel is alive!";

    let mut i = 0;

    while i < message.len() {
        unsafe {
            *vga.offset(i as isize * 2) = message[i];
            *vga.offset(i as isize * 2 + 1) = 0x0F;
        }

        i += 1;
    }

    interrupts::init_idt();
    interrupts::init_pics();

    unsafe {
        core::arch::asm!("sti");
    }

    fb::draw_test_pattern();
    fb::draw_text(10, 10, "Siyana OS v0.1 - framebuffer online", 255, 255, 255);
    fb::draw_text(10, 26, "Type nothing yet. Keyboard IRQ works.", 255, 255, 0);
    memory::probe_and_print();
    heap_self_test();

    loop{}
}

/// Bring up the heap and prove alloc + free both work, on screen.
fn heap_self_test() {
    use alloc::{boxed::Box, format, string::String, vec::Vec};

    let Some(start) = allocator::init() else {
        fb::draw_text(10, 180, "heap: out of memory", 255, 0, 0);
        return;
    };
    fb::draw_text(10, 180, &format!("heap: {} KiB @ {:#x}", allocator::HEAP_SIZE / 1024, start), 0, 255, 0);

    let b = Box::new(41u64);
    let v: Vec<u64> = (1..=1000).collect();
    let mut s = String::from("Siyana");
    s.push_str(" + alloc");
    fb::draw_text(10, 196, &format!("Box={} Vec sum={} String=\"{}\"", *b + 1, v.iter().sum::<u64>(), s), 0, 255, 0);

    // 10,000 x 1 KiB = ~10 MiB through a 4 MiB heap: only works if free() really frees.
    for i in 0..10_000u32 {
        let chunk = alloc::vec![i as u8; 1024];
        core::hint::black_box(&chunk);
    }
    drop((b, v, s));
    let (used, free) = allocator::stats();
    fb::draw_text(10, 212, &format!("10 MiB churned, now used={} B free={} KiB", used, free / 1024), 0, 255, 0);
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop{}
}
