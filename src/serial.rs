//! Serial port (COM1) driver for kernel logging.
//!
//! COM1 is a 16550-compatible UART at I/O port 0x3F8. QEMU forwards it to
//! the terminal with `-serial stdio`, so `serial_println!` lands in the shell
//! you ran `make run` from. On real hardware it goes out the serial cable.

use core::fmt;

use spin::Mutex;
use x86_64::instructions::{interrupts, port::Port};

const COM1: u16 = 0x3F8;

// Register offsets from the base port.
const DATA: u16 = 0; // read/write byte (DLAB=0); divisor low byte (DLAB=1)
const INT_ENABLE: u16 = 1; // interrupt enable (DLAB=0); divisor high byte (DLAB=1)
const FIFO_CTRL: u16 = 2;
const LINE_CTRL: u16 = 3;
const MODEM_CTRL: u16 = 4;
const LINE_STATUS: u16 = 5;

const LSR_TX_EMPTY: u8 = 1 << 5; // transmit holding register empty

pub struct Serial {
    base: u16,
    present: bool,
}

impl Serial {
    const fn new(base: u16) -> Self {
        Serial { base, present: false }
    }

    fn port(&self, offset: u16) -> Port<u8> {
        Port::new(self.base + offset)
    }

    /// Program the UART for 38400 baud, 8 data bits, no parity, 1 stop bit,
    /// then check it with a loopback test. Returns whether a UART answered.
    fn init(&mut self) -> bool {
        unsafe {
            self.port(INT_ENABLE).write(0x00); // no UART interrupts; we poll
            self.port(LINE_CTRL).write(0x80); // DLAB=1: next two writes set the divisor
            self.port(DATA).write(0x03); // divisor 3 -> 115200 / 3 = 38400 baud
            self.port(INT_ENABLE).write(0x00);
            self.port(LINE_CTRL).write(0x03); // DLAB=0, 8N1
            self.port(FIFO_CTRL).write(0xC7); // enable + clear FIFOs, 14-byte threshold
            self.port(MODEM_CTRL).write(0x1E); // loopback mode for the self-test

            // Anything we send in loopback mode comes straight back.
            self.port(DATA).write(0xAE);
            if self.port(DATA).read() != 0xAE {
                self.present = false;
                return false;
            }

            self.port(MODEM_CTRL).write(0x0F); // normal mode: DTR, RTS, OUT1, OUT2
        }
        self.present = true;
        true
    }

    fn write_byte(&mut self, b: u8) {
        if !self.present {
            return;
        }
        unsafe {
            while self.port(LINE_STATUS).read() & LSR_TX_EMPTY == 0 {
                core::hint::spin_loop();
            }
            self.port(DATA).write(b);
        }
    }
}

impl fmt::Write for Serial {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        for b in s.bytes() {
            if b == b'\n' {
                self.write_byte(b'\r'); // terminals want CR+LF
            }
            self.write_byte(b);
        }
        Ok(())
    }
}

pub static SERIAL1: Mutex<Serial> = Mutex::new(Serial::new(COM1));

/// Set up COM1. Returns false if no UART was found (logging becomes a no-op).
pub fn init() -> bool {
    interrupts::without_interrupts(|| SERIAL1.lock().init())
}

#[doc(hidden)]
pub fn _print(args: fmt::Arguments) {
    use core::fmt::Write;
    // Same rule as the heap: never hold this lock with interrupts on.
    interrupts::without_interrupts(|| {
        let _ = SERIAL1.lock().write_fmt(args);
    });
}

/// For panics and fatal exceptions only: write even if the lock is held,
/// because whoever held it is never coming back.
pub fn _print_emergency(args: fmt::Arguments) {
    use core::fmt::Write;
    unsafe { SERIAL1.force_unlock() };
    let _ = SERIAL1.lock().write_fmt(args);
}

#[macro_export]
macro_rules! serial_print {
    ($($arg:tt)*) => { $crate::serial::_print(format_args!($($arg)*)) };
}

#[macro_export]
macro_rules! serial_println {
    () => { $crate::serial_print!("\n") };
    ($($arg:tt)*) => { $crate::serial_print!("{}\n", format_args!($($arg)*)) };
}
