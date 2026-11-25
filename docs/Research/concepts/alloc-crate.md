# alloc crate

The part of Rust's standard library that needs a heap but no OS: Box, Vec, String, BTreeMap, format!. A no_std kernel enables it with `extern crate alloc;` once it has a global allocator.
