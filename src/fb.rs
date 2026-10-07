//! VESA linear framebuffer driver.
//!
//! The bootloader (stage 2) switches the GPU into VBE mode 0x112
//! (640x480, 24-bit color) and leaves the linear framebuffer address,
//! pitch, width and height at physical address 0x7C00.

use core::ptr;

const FB_INFO_BASE: *const u8 = 0x7C00 as *const u8;

fn read_u32(offset: usize) -> u32 {
    unsafe { ptr::read_volatile(FB_INFO_BASE.add(offset) as *const u32) }
}

fn read_u16(offset: usize) -> u16 {
    unsafe { ptr::read_volatile(FB_INFO_BASE.add(offset) as *const u16) }
}

pub fn address() -> usize {
    read_u32(0x00) as usize
}

pub fn pitch() -> usize {
    read_u16(0x04) as usize
}

pub fn width() -> usize {
    read_u16(0x08) as usize
}

pub fn height() -> usize {
    read_u16(0x0A) as usize
}

/// Fill the screen with a color given as (R, G, B).
pub fn clear(r: u8, g: u8, b: u8) {
    let lfb = address() as *mut u8;
    let (w, h, p) = (width(), height(), pitch());

    for y in 0..h {
        for x in 0..w {
            let px = unsafe { lfb.add(y * p + x * 3) };
            unsafe {
                *px = b;
                *px.add(1) = g;
                *px.add(2) = r;
            }
        }
    }
}

/// Plot a single pixel.
pub fn put_pixel(x: usize, y: usize, r: u8, g: u8, b: u8) {
    if x >= width() || y >= height() {
        return;
    }
    let lfb = address() as *mut u8;
    let p = pitch();
    let px = unsafe { lfb.add(y * p + x * 3) };
    unsafe {
        *px = b;
        *px.add(1) = g;
        *px.add(2) = r;
    }
}

/// Draw a simple gradient as a visual proof that the framebuffer works.
pub fn draw_test_pattern() {
    let (w, h) = (width(), height());
    for y in 0..h {
        for x in 0..w {
            let r = (x * 255 / w) as u8;
            let g = (y * 255 / h) as u8;
            put_pixel(x, y, r, g, 0x30);
        }
    }
}
