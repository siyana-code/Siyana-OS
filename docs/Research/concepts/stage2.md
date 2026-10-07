# Stage 2 bootloader

A bigger loader (boot/stage2.asm) loaded by stage 1. It reads the kernel from disk, sets up a GDT, turns on protected mode, builds page tables, enters long mode, and finally jumps to _start in the Rust kernel.
