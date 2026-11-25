//! Command-line arguments and environment variables.

use alloc::string::String;
use alloc::vec::Vec;

use crate::backend;

/// All arguments, including the program name at index 0.
pub fn args() -> Vec<String> {
    backend::args()
}

/// All environment variables as `(key, value)` pairs.
pub fn vars() -> Vec<(String, String)> {
    backend::env()
}

pub fn var(key: &str) -> Option<String> {
    backend::env().into_iter().find(|(k, _)| k == key).map(|(_, v)| v)
}
