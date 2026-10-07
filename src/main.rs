#![no_std]
#![no_main]
#![feature(abi_x86_interrupt)]

mod interrupts;
mod fb;

use core::panic::PanicInfo;

#[unsafe(no_mangle)]
#[unsafe(link_section = ".text.start")]
extern "C" fn _start() -> ! {
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

    loop{}
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop{}
}