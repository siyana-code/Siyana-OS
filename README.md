# Siyana OS

A hobby operating system built entirely from scratch in Rust and x86 assembly, as a hands-on learning project.

## Current status

Siyana OS currently boots on real x86_64 hardware (tested in QEMU) through the following chain:

1. **Stage 1 bootloader** (`boot/boot.asm`) — a 512-byte BIOS boot sector that prints a startup message and loads Stage 2 from disk.
2. **Stage 2 bootloader** (`boot/stage2.asm`) — loads the Rust kernel from disk, sets up a Global Descriptor Table (GDT), switches the CPU from 16-bit real mode to 32-bit protected mode, builds minimal page tables, switches to 64-bit long mode, and jumps into the kernel.
3. **Kernel** (`src/main.rs`) — a freestanding (`no_std`, `no_main`) Rust binary, linked to a fixed load address via a custom linker script (`linker.ld`), currently printing a startup message directly to VGA text-mode memory.

## Requirements

- Rust (nightly toolchain, via `rustup`)
- `x86_64-unknown-none` Rust target
- NASM (assembler)
- QEMU (`qemu-system-x86_64`)
- Standard binutils (`objcopy`, `readelf`, etc.)

## Building and running

```bash
make run
```

This assembles the bootloader stages, builds the kernel in release mode (required — debug builds include runtime safety checks incompatible with a freestanding environment), links it at a fixed address, assembles a raw disk image, and boots it in QEMU.

```bash
make clean
```

Removes all build artifacts.

## Project structure

```
boot/
  boot.asm      - Stage 1: BIOS boot sector
  stage2.asm    - Stage 2: protected/long mode setup, kernel loader
src/
  main.rs       - Kernel entry point
  interrupts.rs - IDT, PIC setup, exception/IRQ handlers
linker.ld       - Kernel linker script (fixed load address 0x10000)
Makefile        - Build automation
docs/
  NOTES.md      - Teaching notes per subsystem (what/why/how)
  PLAN.md       - Production project plan and branching model
```

## Roadmap

See [docs/PLAN.md](docs/PLAN.md) for the full production roadmap, and
[docs/NOTES.md](docs/NOTES.md) for teaching notes on each subsystem.

- [x] Interrupt handling (IDT)
- [x] Keyboard IRQ handler (scancode draining; key decoding pending)
- [ ] Proper VGA text driver (scrolling, screen clearing)
- [ ] Memory management (physical/virtual allocators)
- [ ] Basic task/process model