//! `libsiyana`: the Siyana OS userspace API.
//!
//! Apps use this crate for everything that touches the OS: console, files,
//! time, processes, events and IPC. It has two backends behind one API:
//!
//! - **Linux** (`target_os = "linux"`): built on `std` + `libc`, so apps
//!   build, run and are tested on Ubuntu today.
//! - **Siyana** (`target_os = "none"`): native Siyana system calls
//!   (see `siyana-abi` and `docs/abi.md`). Compiles today; runs once the
//!   kernel implements the calls (Phase 4).
//!
//! # Writing a portable app
//!
//! ```ignore
//! #![cfg_attr(not(target_os = "linux"), no_std, no_main)]
//! use libsiyana::prelude::*;
//!
//! libsiyana::entry!(app_main);
//!
//! fn app_main() -> i32 {
//!     println!("hello from {}", libsiyana::process::id());
//!     0
//! }
//! ```
//!
//! Rule of thumb: use `core`, `alloc` and `libsiyana`, never `std`. Check
//! with `cargo check --target x86_64-unknown-none`: if it compiles there,
//! it will run on Siyana.

#![cfg_attr(not(target_os = "linux"), no_std)]

extern crate alloc;

mod backend;
mod error;

pub mod env;
pub mod event;
pub mod fs;
pub mod io;
pub mod ipc;
pub mod process;
pub mod time;

#[cfg(not(target_os = "linux"))]
#[doc(hidden)]
pub mod rt;

pub use error::{Error, Result};
pub use siyana_abi as abi;

/// Everyday imports: `String`, `Vec`, `Box`, `format!`, `vec!`, `ToString`,
/// `Duration` and the print macros.
pub mod prelude {
    // On Linux, std's prelude already has these (writing to the same
    // stdout/stderr); re-exporting ours there would be ambiguous.
    #[cfg(not(target_os = "linux"))]
    pub use crate::{eprint, eprintln, print, println};
    pub use alloc::borrow::ToOwned;
    pub use alloc::boxed::Box;
    pub use alloc::string::{String, ToString};
    pub use alloc::vec::Vec;
    pub use alloc::{format, vec};
    pub use core::time::Duration;
}

/// A kernel object owned by this process (file, IPC stream, console...).
pub type Handle = u32;

/// Declare the app's entry point: `fn() -> i32`, the exit code.
///
/// On Linux this becomes `fn main`. On Siyana, `libsiyana`'s `_start`
/// calls it and exits with the returned code.
#[macro_export]
macro_rules! entry {
    ($main:path) => {
        #[cfg(target_os = "linux")]
        fn main() {
            let f: fn() -> i32 = $main;
            $crate::process::exit(f())
        }

        #[cfg(not(target_os = "linux"))]
        #[unsafe(no_mangle)]
        pub fn __siyana_main() -> i32 {
            let f: fn() -> i32 = $main;
            f()
        }
    };
}
