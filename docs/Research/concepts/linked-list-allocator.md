# Linked-list allocator

Keeps a list of free holes inside the heap. alloc finds the first hole big enough (first fit) and splits it; dealloc puts the piece back and merges it with neighbouring holes. Simple and it really frees, unlike the bump allocator. We use the linked_list_allocator crate.
