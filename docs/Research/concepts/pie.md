# PIE (position-independent executable)

A program that can run at any address. The loader picks a base address and patches a list of pointers (R_X86_64_RELATIVE relocations). Rust builds PIE by default for x86_64-unknown-none, so Siyana's loader will accept it.
