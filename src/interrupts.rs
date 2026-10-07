use x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame};
use x86_64::instructions::port::Port;
use spin::{Lazy, Mutex};
use pic8259::ChainedPics;

pub const PIC_1_OFFSET: u8 = 32;
pub const PIC_2_OFFSET: u8 = 32 + 8;

const KEYBOARD_INTERRUPT_ID: u8 = PIC_1_OFFSET + 1;

pub static PICS: Mutex<ChainedPics> = Mutex::new(unsafe {
    ChainedPics::new(PIC_1_OFFSET, PIC_2_OFFSET)
});

static IDT: Lazy<InterruptDescriptorTable> = Lazy::new(|| {
    let mut idt = InterruptDescriptorTable::new();
    idt.divide_error.set_handler_fn(divide_by_zero_handler);
    idt[KEYBOARD_INTERRUPT_ID].set_handler_fn(keyboard_interrupt_handler);
    idt
});

pub fn init_idt() {
    IDT.load();
}

pub fn init_pics() {
    unsafe {
        PICS.lock().initialize()
    };
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
    let _scancode = unsafe { port.read() };

    let vga = 0xB8000 as *mut u8;
    let message = b"KEY PRESSED!";
    let mut i = 0;

    while i < message.len() {
        unsafe {
            *vga.offset(i as isize * 2) = message[i];
            *vga.offset(i as isize * 2 + 1)  = 0x2F;
        }

        i += 1;
    }

    unsafe {
        PICS.lock().notify_end_of_interrupt(KEYBOARD_INTERRUPT_ID);
    }
}