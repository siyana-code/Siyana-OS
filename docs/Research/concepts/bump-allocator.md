# Bump allocator

The simplest allocator: a single pointer that only moves forward. Every allocation returns the next free chunk. Fast and safe to start with; free() is a no-op until we add a real free list.
