use core::fmt;

use siyana_abi::error as code;

/// Everything that can go wrong in a libsiyana call.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Error {
    NotFound,
    PermissionDenied,
    AlreadyExists,
    InvalidInput,
    WouldBlock,
    Interrupted,
    NotADirectory,
    IsADirectory,
    OutOfMemory,
    BadHandle,
    NoChild,
    /// The OS does not support this call (yet).
    Unsupported,
    BrokenPipe,
    TimedOut,
    NotEmpty,
    /// Data was not valid UTF-8.
    InvalidUtf8,
    /// Any other failure, with the raw ABI error code.
    Other(isize),
}

pub type Result<T> = core::result::Result<T, Error>;

impl Error {
    /// Map an ABI error code (positive) to an `Error`.
    pub fn from_code(c: isize) -> Error {
        match c {
            code::NOT_FOUND => Error::NotFound,
            code::PERMISSION_DENIED => Error::PermissionDenied,
            code::ALREADY_EXISTS => Error::AlreadyExists,
            code::INVALID_INPUT => Error::InvalidInput,
            code::WOULD_BLOCK => Error::WouldBlock,
            code::INTERRUPTED => Error::Interrupted,
            code::NOT_A_DIRECTORY => Error::NotADirectory,
            code::IS_A_DIRECTORY => Error::IsADirectory,
            code::OUT_OF_MEMORY => Error::OutOfMemory,
            code::BAD_HANDLE => Error::BadHandle,
            code::NO_CHILD => Error::NoChild,
            code::UNSUPPORTED => Error::Unsupported,
            code::BROKEN_PIPE => Error::BrokenPipe,
            code::TIMED_OUT => Error::TimedOut,
            code::NOT_EMPTY => Error::NotEmpty,
            other => Error::Other(other),
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let s = match self {
            Error::NotFound => "not found",
            Error::PermissionDenied => "permission denied",
            Error::AlreadyExists => "already exists",
            Error::InvalidInput => "invalid input",
            Error::WouldBlock => "operation would block",
            Error::Interrupted => "interrupted",
            Error::NotADirectory => "not a directory",
            Error::IsADirectory => "is a directory",
            Error::OutOfMemory => "out of memory",
            Error::BadHandle => "bad handle",
            Error::NoChild => "no child process",
            Error::Unsupported => "not supported",
            Error::BrokenPipe => "broken pipe",
            Error::TimedOut => "timed out",
            Error::NotEmpty => "directory not empty",
            Error::InvalidUtf8 => "invalid UTF-8",
            Error::Other(c) => return write!(f, "error code {c}"),
        };
        f.write_str(s)
    }
}

impl core::error::Error for Error {}
