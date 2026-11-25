//! Local inter-process communication: named byte streams between processes
//! on the same machine (like Unix domain sockets).
//!
//! A server calls [`Listener::bind`] with a name; clients call
//! [`Stream::connect`] with the same name. Names are 1-64 characters of
//! `A-Z a-z 0-9 . _ -`.

use alloc::string::{String, ToString};
use alloc::vec::Vec;

use crate::{backend, io, Error, Handle, Result};

fn check_name(name: &str) -> Result<()> {
    let ok = !name.is_empty()
        && name.len() <= 64
        && name.bytes().all(|b| b.is_ascii_alphanumeric() || b"._-".contains(&b));
    if ok { Ok(()) } else { Err(Error::InvalidInput) }
}

/// A named endpoint waiting for connections. Unregistered when dropped.
#[derive(Debug)]
pub struct Listener {
    handle: Handle,
    name: String,
}

impl Listener {
    /// Register `name`. Fails with `AlreadyExists` if a live listener has it.
    pub fn bind(name: &str) -> Result<Listener> {
        check_name(name)?;
        Ok(Listener { handle: backend::ipc_listen(name)?, name: name.to_string() })
    }

    /// Wait for the next client and return a stream to it.
    pub fn accept(&self) -> Result<Stream> {
        Ok(Stream { handle: backend::ipc_accept(self.handle)? })
    }

    /// Watch this with `event::wait`: `Readable` means a client is waiting.
    pub fn handle(&self) -> Handle {
        self.handle
    }

    pub fn name(&self) -> &str {
        &self.name
    }
}

impl Drop for Listener {
    fn drop(&mut self) {
        backend::ipc_unlisten(self.handle, &self.name);
    }
}

/// A connected two-way byte stream. Closed when dropped.
#[derive(Debug)]
pub struct Stream {
    handle: Handle,
}

impl Stream {
    /// Connect to the listener registered as `name`.
    pub fn connect(name: &str) -> Result<Stream> {
        check_name(name)?;
        Ok(Stream { handle: backend::ipc_connect(name)? })
    }

    pub fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        io::read(self.handle, buf)
    }

    /// Read until the other side closes.
    pub fn read_to_end(&mut self, out: &mut Vec<u8>) -> Result<usize> {
        io::read_to_end(self.handle, out)
    }

    pub fn write(&mut self, buf: &[u8]) -> Result<usize> {
        io::write(self.handle, buf)
    }

    pub fn write_all(&mut self, buf: &[u8]) -> Result<()> {
        io::write_all(self.handle, buf)
    }

    /// Stop sending; the other side's reads then return 0 (end of input).
    pub fn shutdown_write(&mut self) -> Result<()> {
        backend::ipc_shutdown_write(self.handle)
    }

    pub fn handle(&self) -> Handle {
        self.handle
    }
}

impl Drop for Stream {
    fn drop(&mut self) {
        let _ = backend::close(self.handle);
    }
}
