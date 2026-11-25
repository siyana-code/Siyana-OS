# Global allocator

The one allocator Rust's alloc crate calls for every Box/Vec/String. You pick it with #[global_allocator] on a static that implements GlobalAlloc (alloc + dealloc). Ours is the HEAP static in src/allocator.rs.
