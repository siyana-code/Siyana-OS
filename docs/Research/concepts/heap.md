# Heap

A pool of memory the kernel hands out in any size and takes back when done (Box, Vec, String). Ours is 4 MiB of physical frames from the frame allocator, set up by allocator::init() in src/allocator.rs.
