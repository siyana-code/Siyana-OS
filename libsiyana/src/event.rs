//! The per-process event queue: Siyana's replacement for POSIX signals.
//!
//! Instead of handlers that interrupt your code at random points, things
//! that happen to the process are queued, and you collect them when you're
//! ready, in your main loop:
//!
//! ```ignore
//! loop {
//!     for ev in event::wait(Some(Duration::from_secs(1)), &[listener.handle()])? {
//!         match ev {
//!             Event::Shutdown => return stop_everything(),
//!             Event::ChildExited => for (pid, st) in process::reap() { /* ... */ },
//!             Event::Readable(_) => handle_client(listener.accept()?),
//!         }
//!     }
//! }
//! ```
//!
//! The first call to `wait` opts the process in: from then on
//! `Signal::Terminate` (and Ctrl-C on a terminal) arrives as
//! `Event::Shutdown` instead of killing it.

use alloc::vec::Vec;
use core::time::Duration;

use crate::{backend, Handle, Result};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Event {
    /// Someone asked this process to stop.
    Shutdown,
    /// At least one child finished: collect them with `process::reap`.
    ChildExited,
    /// A watched handle has data, or a listener has a connection waiting.
    Readable(Handle),
}

/// Wait until something happens, for at most `timeout` (`None` = forever,
/// `Some(Duration::ZERO)` = just check). Returns every pending event; an
/// empty list means the timeout passed.
pub fn wait(timeout: Option<Duration>, watch: &[Handle]) -> Result<Vec<Event>> {
    let ns = match timeout {
        None => u64::MAX,
        Some(d) => u64::try_from(d.as_nanos()).unwrap_or(u64::MAX - 1),
    };
    backend::event_wait(ns, watch)
}
