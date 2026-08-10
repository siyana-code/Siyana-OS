#![no_std]
#![no_main]

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
    loop{}
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop{}
}