//! Console and raw handle I/O.

use alloc::string::String;
use alloc::vec::Vec;
use core::fmt;

use siyana_abi::handle::{STDERR, STDIN, STDOUT};

use crate::{backend, Error, Handle, Result};

/// Read up to `buf.len()` bytes from a handle. 0 means end of input.
pub fn read(h: Handle, buf: &mut [u8]) -> Result<usize> {
    backend::read(h, buf)
}

/// Write some of `buf` to a handle; returns how many bytes went out.
pub fn write(h: Handle, buf: &[u8]) -> Result<usize> {
    backend::write(h, buf)
}

/// Write all of `buf`, retrying short writes.
pub fn write_all(h: Handle, mut buf: &[u8]) -> Result<()> {
    while !buf.is_empty() {
        match backend::write(h, buf) {
            Ok(0) => return Err(Error::BrokenPipe),
            Ok(n) => buf = &buf[n..],
            Err(Error::Interrupted) => {}
            Err(e) => return Err(e),
        }
    }
    Ok(())
}

/// Read until end of input.
pub fn read_to_end(h: Handle, out: &mut Vec<u8>) -> Result<usize> {
    let start = out.len();
    let mut chunk = [0u8; 4096];
    loop {
        match backend::read(h, &mut chunk) {
            Ok(0) => return Ok(out.len() - start),
            Ok(n) => out.extend_from_slice(&chunk[..n]),
            Err(Error::Interrupted) => {}
            Err(e) => return Err(e),
        }
    }
}

/// Read one line from stdin into `line`, including the trailing `\n` if
/// there was one. Returns the number of bytes read; 0 means end of input.
pub fn read_line(line: &mut String) -> Result<usize> {
    let mut bytes = Vec::new();
    let mut b = [0u8; 1];
    loop {
        match backend::read(STDIN, &mut b) {
            Ok(0) => break,
            Ok(_) => {
                bytes.push(b[0]);
                if b[0] == b'\n' {
                    break;
                }
            }
            Err(Error::Interrupted) => {}
            Err(e) => return Err(e),
        }
    }
    let s = core::str::from_utf8(&bytes).map_err(|_| Error::InvalidUtf8)?;
    line.push_str(s);
    Ok(bytes.len())
}

/// `fmt::Write` adapter over a handle.
pub struct HandleWriter(pub Handle);

impl fmt::Write for HandleWriter {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        write_all(self.0, s.as_bytes()).map_err(|_| fmt::Error)
    }
}

#[doc(hidden)]
pub fn _print(err: bool, args: fmt::Arguments) {
    use fmt::Write;
    let h = if err { STDERR } else { STDOUT };
    let _ = HandleWriter(h).write_fmt(args);
}

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => { $crate::io::_print(false, format_args!($($arg)*)) };
}

#[macro_export]
macro_rules! println {
    () => { $crate::print!("\n") };
    ($($arg:tt)*) => { $crate::io::_print(false, format_args!("{}\n", format_args!($($arg)*))) };
}

#[macro_export]
macro_rules! eprint {
    ($($arg:tt)*) => { $crate::io::_print(true, format_args!($($arg)*)) };
}

#[macro_export]
macro_rules! eprintln {
    () => { $crate::eprint!("\n") };
    ($($arg:tt)*) => { $crate::io::_print(true, format_args!("{}\n", format_args!($($arg)*))) };
}
