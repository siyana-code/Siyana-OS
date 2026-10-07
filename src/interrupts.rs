use x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame};
use x86_64::instructions::port::Port;
use spin::{Lazy, Mutex};
use pic8259::ChainedPics;
use core::sync::atomic::{AtomicU64, Ordering};

static TICKS: AtomicU64 = AtomicU64::new(0);

pub const PIC_1_OFFSET: u8 = 32;
pub const PIC_2_OFFSET: u8 = 32 + 8;

const KEYBOARD_INTERRUPT_ID: u8 = PIC_1_OFFSET + 1;
const TIMER_INTERRUPT_ID: u8 = PIC_1_OFFSET;

pub static PICS: Mutex<ChainedPics> = Mutex::new(unsafe {
    ChainedPics::new(PIC_1_OFFSET, PIC_2_OFFSET)
});

static IDT: Lazy<InterruptDescriptorTable> = Lazy::new(|| {
    let mut idt = InterruptDescriptorTable::new();
    idt.divide_error.set_handler_fn(divide_by_zero_handler);
    idt[KEYBOARD_INTERRUPT_ID].set_handler_fn(keyboard_interrupt_handler);
    idt[TIMER_INTERRUPT_ID].set_handler_fn(timer_interrupt_handler);
    idt
});

pub fn init_idt() {
    IDT.load();
}

pub fn init_pics() {
    unsafe {
        let mut pics = PICS.lock();
        pics.initialize();
        // Unmask IRQ0 (timer) and IRQ1 (keyboard); mask everything else.
        pics.write_masks(0b1111_1100, 0xFF);
    };
    // Program the PIT (8253) channel 0: 100 Hz tick.
    // Divisor = 1193182 / 100 ≈ 11932.
    unsafe {
        let mut cmd = Port::<u8>::new(0x43);
        let mut ch0 = Port::<u8>::new(0x40);
        cmd.write(0x36); // channel 0, lobyte/hibyte, mode 3 (square wave)
        ch0.write((11932 & 0xFF) as u8);
        ch0.write((11932 >> 8) as u8);
    }
}

extern "x86-interrupt" fn divide_by_zero_handler(_stack_frame: InterruptStackFrame) {
    let vga = 0xB8000 as *mut u8;
    let message = b"EXCEPTION: DIVIDE BY ZERO!";
    let mut i = 0;

    while i < message.len() {
        unsafe {
            *vga.offset(i as isize * 2) = message[i];
            *vga.offset(i as isize * 2 + 1) = 0x4F;
        }

        i += 1;
    }

    loop {}
}

extern "x86-interrupt" fn keyboard_interrupt_handler(_stack_frame: InterruptStackFrame) {
    // Drain the scancode from the keyboard controller (port 0x60).
    // Without this the controller's output buffer stays full and no
    // further IRQs are delivered.
    let mut port = Port::<u8>::new(0x60);
    let scancode = unsafe { port.read() };
    crate::keyboard::handle_scancode(scancode);

    unsafe {
        PICS.lock().notify_end_of_interrupt(KEYBOARD_INTERRUPT_ID);
    }
}

extern "x86-interrupt" fn timer_interrupt_handler(_stack_frame: InterruptStackFrame) {
    let ticks = TICKS.fetch_add(1, Ordering::SeqCst) + 1;

    // Once per second (100 Hz) update the on-screen counter.
    if ticks % 100 == 0 {
        let mut buf = [b' '; 24];
        let prefix = b"Ticks: ";
        buf[..prefix.len()].copy_from_slice(prefix);
        let mut n = ticks / 100;
        let mut digits = [0u8; 20];
        let mut len = 0;
        if n == 0 {
            digits[0] = b'0';
            len = 1;
        } else {
            while n > 0 {
                digits[len] = b'0' + (n % 10) as u8;
                n /= 10;
                len += 1;
            }
        }
        for i in 0..len {
            buf[prefix.len() + i] = digits[len - 1 - i];
        }
        // Clear the counter band first so digits never leave residue.
        for y in 66..74 {
            for x in 10..10 + 24 * 8 {
                crate::fb::put_pixel(x, y, 0x20, 0x20, 0x30);
            }
        }
        if let Ok(s) = core::str::from_utf8(&buf) {
            crate::fb::draw_text(10, 66, s, 0, 255, 0);
        }
    }

    unsafe {
        PICS.lock().notify_end_of_interrupt(TIMER_INTERRUPT_ID);
    }
}