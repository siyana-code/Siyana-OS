//! Keyboard scancode handling (PS/2 set 1).
//!
//! The IRQ1 handler in `interrupts.rs` drains the raw scancode from
//! port 0x60 and hands it here. We translate scancodes to ASCII and
//! echo them onto the framebuffer — the kernel's first real input.

use core::sync::atomic::{AtomicUsize, Ordering};

use crate::fb;

static ECHO_COL: AtomicUsize = AtomicUsize::new(10);
const ECHO_ROW: usize = 50;
const ECHO_MAX_COL: usize = 75;

/// US QWERTY scancode set 1 → ASCII (make codes only).
fn scancode_to_ascii(code: u8) -> Option<char> {
    let c = match code {
        0x02 => '1', 0x03 => '2', 0x04 => '3', 0x05 => '4', 0x06 => '5',
        0x07 => '6', 0x08 => '7', 0x09 => '8', 0x0A => '9', 0x0B => '0',
        0x0C => '-', 0x0D => '=',
        0x10 => 'q', 0x11 => 'w', 0x12 => 'e', 0x13 => 'r', 0x14 => 't',
        0x15 => 'y', 0x16 => 'u', 0x17 => 'i', 0x18 => 'o', 0x19 => 'p',
        0x1E => 'a', 0x1F => 's', 0x20 => 'd', 0x21 => 'f', 0x22 => 'g',
        0x23 => 'h', 0x24 => 'j', 0x25 => 'k', 0x26 => 'l',
        0x2C => 'z', 0x2D => 'x', 0x2E => 'c', 0x2F => 'v', 0x30 => 'b',
        0x31 => 'n', 0x32 => 'm',
        0x39 => ' ',
        _ => return None,
    };
    Some(c)
}

pub fn handle_scancode(code: u8) {
    // Bit 7 set = key release; ignore releases for now.
    if code & 0x80 != 0 {
        return;
    }
    let Some(c) = scancode_to_ascii(code) else { return };

    let mut col = ECHO_COL.load(Ordering::SeqCst);
    let mut buf = [0u8; 4];
    let s = c.encode_utf8(&mut buf);
    fb::draw_text(col * 8, ECHO_ROW, s, 0, 255, 255);

    col += 1;
    if col > ECHO_MAX_COL {
        col = 10;
        // simple "clear line": overdraw background color band
        for y in ECHO_ROW..ECHO_ROW + 8 {
            for x in 10 * 8..(ECHO_MAX_COL + 1) * 8 {
                fb::put_pixel(x, y, 0x20, 0x20, 0x30);
            }
        }
    }
    ECHO_COL.store(col, Ordering::SeqCst);
}
