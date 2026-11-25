//! OS backends. Each one provides the same set of functions, one per ABI
//! call, with Rust types; the public modules are written once on top.

#[cfg(target_os = "linux")]
mod linux;
#[cfg(target_os = "linux")]
pub(crate) use linux::*;

#[cfg(not(target_os = "linux"))]
mod siyana;
#[cfg(not(target_os = "linux"))]
pub(crate) use siyana::*;
