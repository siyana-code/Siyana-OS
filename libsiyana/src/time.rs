//! Clocks and sleeping.

pub use core::time::Duration;

use siyana_abi::clock;

use crate::backend;

/// A point on the monotonic clock (never goes backwards).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Instant(u64);

impl Instant {
    pub fn now() -> Instant {
        Instant(backend::clock(clock::MONOTONIC))
    }

    /// Time since `earlier`; zero if `earlier` is actually later.
    pub fn duration_since(&self, earlier: Instant) -> Duration {
        Duration::from_nanos(self.0.saturating_sub(earlier.0))
    }

    pub fn elapsed(&self) -> Duration {
        Instant::now().duration_since(*self)
    }

    pub fn checked_add(&self, d: Duration) -> Option<Instant> {
        u64::try_from(d.as_nanos()).ok().and_then(|n| self.0.checked_add(n)).map(Instant)
    }
}

impl core::ops::Add<Duration> for Instant {
    type Output = Instant;
    fn add(self, d: Duration) -> Instant {
        self.checked_add(d).expect("Instant overflow")
    }
}

impl core::ops::Sub for Instant {
    type Output = Duration;
    fn sub(self, earlier: Instant) -> Duration {
        self.duration_since(earlier)
    }
}

/// Wall-clock time since 1970-01-01 UTC.
pub fn unix_time() -> Duration {
    Duration::from_nanos(backend::clock(clock::REALTIME))
}

pub fn sleep(d: Duration) {
    backend::sleep(u64::try_from(d.as_nanos()).unwrap_or(u64::MAX));
}
