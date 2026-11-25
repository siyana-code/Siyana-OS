//! Files and directories. Paths are UTF-8 strings.

use alloc::string::String;
use alloc::vec::Vec;

use siyana_abi::{mkdir, open, remove};

use crate::{backend, io, Error, Handle, Result};

/// An open file. Closed when dropped.
#[derive(Debug)]
pub struct File {
    handle: Handle,
}

impl File {
    /// Open an existing file for reading.
    pub fn open(path: &str) -> Result<File> {
        OpenOptions::new().read(true).open(path)
    }

    /// Create (or truncate) a file for writing.
    pub fn create(path: &str) -> Result<File> {
        OpenOptions::new().write(true).create(true).truncate(true).open(path)
    }

    pub fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        io::read(self.handle, buf)
    }

    pub fn read_to_end(&mut self, out: &mut Vec<u8>) -> Result<usize> {
        io::read_to_end(self.handle, out)
    }

    pub fn write(&mut self, buf: &[u8]) -> Result<usize> {
        io::write(self.handle, buf)
    }

    pub fn write_all(&mut self, buf: &[u8]) -> Result<()> {
        io::write_all(self.handle, buf)
    }

    pub fn handle(&self) -> Handle {
        self.handle
    }
}

impl Drop for File {
    fn drop(&mut self) {
        let _ = backend::close(self.handle);
    }
}

/// Builder for opening files with specific modes.
#[derive(Debug, Clone, Copy, Default)]
pub struct OpenOptions {
    flags: usize,
}

impl OpenOptions {
    pub fn new() -> Self {
        Self::default()
    }

    fn set(mut self, flag: usize, on: bool) -> Self {
        if on { self.flags |= flag } else { self.flags &= !flag }
        self
    }

    pub fn read(self, on: bool) -> Self { self.set(open::READ, on) }
    pub fn write(self, on: bool) -> Self { self.set(open::WRITE, on) }
    pub fn create(self, on: bool) -> Self { self.set(open::CREATE, on) }
    pub fn truncate(self, on: bool) -> Self { self.set(open::TRUNCATE, on) }
    pub fn append(self, on: bool) -> Self { self.set(open::APPEND | open::WRITE, on) }

    pub fn open(&self, path: &str) -> Result<File> {
        Ok(File { handle: backend::open(path, self.flags)? })
    }
}

/// Read a whole file.
pub fn read(path: &str) -> Result<Vec<u8>> {
    let mut out = Vec::new();
    File::open(path)?.read_to_end(&mut out)?;
    Ok(out)
}

/// Read a whole UTF-8 file.
pub fn read_to_string(path: &str) -> Result<String> {
    String::from_utf8(read(path)?).map_err(|_| Error::InvalidUtf8)
}

/// Create or replace a file with `data`.
pub fn write(path: &str, data: impl AsRef<[u8]>) -> Result<()> {
    File::create(path)?.write_all(data.as_ref())
}

pub fn create_dir(path: &str) -> Result<()> {
    backend::mkdir(path, 0)
}

/// Create a directory and any missing parents. Existing is fine.
pub fn create_dir_all(path: &str) -> Result<()> {
    backend::mkdir(path, mkdir::PARENTS)
}

pub fn remove_file(path: &str) -> Result<()> {
    backend::remove(path, 0)
}

/// Remove an empty directory.
pub fn remove_dir(path: &str) -> Result<()> {
    backend::remove(path, remove::DIRECTORY)
}

/// Remove a directory and everything in it.
pub fn remove_dir_all(path: &str) -> Result<()> {
    backend::remove(path, remove::DIRECTORY | remove::RECURSIVE)
}

/// Does `path` exist?
pub fn exists(path: &str) -> bool {
    match backend::open(path, open::READ) {
        Ok(h) => {
            let _ = backend::close(h);
            true
        }
        Err(Error::IsADirectory) => true,
        Err(_) => backend::open(path, open::DIRECTORY).map(|h| backend::close(h)).is_ok(),
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FileType {
    File,
    Directory,
    Other,
}

/// One entry of a directory listing.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DirEntry {
    pub name: String,
    pub file_type: FileType,
}

/// List a directory's entries (not including `.` and `..`), sorted by name.
pub fn read_dir(path: &str) -> Result<Vec<DirEntry>> {
    let mut entries = backend::read_dir(path)?;
    entries.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(entries)
}
