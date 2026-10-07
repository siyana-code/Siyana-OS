# Atomic counter

A number updated safely from interrupt handlers and normal code without locks, using core::sync::atomic. We keep TICKS: AtomicU64.
