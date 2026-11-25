# Research — Concepts Glossary

Simple, donkey-friendly explanation of every keyword and concept in Siyana OS.
Each word links to its own file.

## Boot & Firmware
- [BIOS](concepts/bios.md) — the firmware that starts your PC
- [Boot sector / Stage 1](concepts/boot-sector.md) — first 512 bytes on disk
- [Stage 2 bootloader](concepts/stage2.md) — the loader that sets up CPU modes
- [Disk image](concepts/disk-image.md) — the fake hard disk QEMU boots from
- [Sector](concepts/sector.md) — a 512-byte slice of a disk
- [Kernel sectors](concepts/kernel-sectors.md) — how big the kernel is, in sectors
- [LBA](concepts/lba.md) — sectors numbered in one straight line
- [INT 13h extensions](concepts/int13-extensions.md) — BIOS reads by LBA
- [Disk Address Packet](concepts/dap.md) — the request we give AH=42h
- [Kernel size limit](concepts/kernel-size-limit.md) — 448 KiB, enforced by the linker
- [NASM](concepts/nasm.md) — the assembler that turns .asm into machine code
- [objcopy](concepts/objcopy.md) — strips ELF into a raw binary
- [QEMU](concepts/qemu.md) — the virtual machine we run on

## CPU Modes & Tables
- [Real mode](concepts/real-mode.md) — the CPU's 16-bit startup brain
- [Protected mode](concepts/protected-mode.md) — 32-bit guarded memory
- [Long mode](concepts/long-mode.md) — 64-bit native mode
- [GDT](concepts/gdt.md) — segment table the CPU needs to switch modes
- [Paging / page tables](concepts/paging.md) — virtual → physical address
     translation
- [PML4 / PDPT / PD / PT](concepts/paging-levels.md) — the 4 table levels
- [Identity mapping](concepts/identity-mapping.md) — virtual == physical
- [CR0 / CR3 / CR4](concepts/control-registers.md) — CPU mode switches
- [EFER / MSR](concepts/efer.md) — extra feature register (enables long mode)
- [Linker script](concepts/linker-script.md) — tells the linker where to put code
- [ELF](concepts/elf.md) — the kernel's file format before objcopy

## Memory & Paging
- [Page fault](concepts/page-fault.md) — exception #14
- [CR2 register](concepts/cr2.md) — faulting address
- [Page-fault error code](concepts/page-fault-error-code.md) — what went wrong
- [Not-present page](concepts/not-present.md) — missing mapping
- [Protection violation](concepts/protection-violation.md) — illegal access

## Interrupts & Timing
- [PIT (8253/8254)](concepts/pit.md) — the clock chip
- [PIT Channel 0](concepts/pit-channel0.md) — drives IRQ0
- [PIT divisor](concepts/pit-divisor.md) — sets the tick frequency
- [PIT mode 3](concepts/pit-mode3.md) — periodic square wave
- [Tick](concepts/tick.md) — one timer interrupt
- [Atomic counter](concepts/atomic-counter.md) — lock-free counter in handlers

## Memory
- [BIOS E820](concepts/e820.md) — BIOS memory map query
- [Memory map](concepts/memory-map.md) — region list of physical RAM
- [Usable region](concepts/usable-region.md) — type-1 E820 entries
- [Bump allocator](concepts/bump-allocator.md) — simplest frame allocator
- [Physical frame](concepts/physical-frame.md) — 4 KiB RAM chunk
- [Page size](concepts/page-size.md) — the 4 KiB unit paging uses
- [Heap](concepts/heap.md) — memory handed out in any size
- [Global allocator](concepts/global-allocator.md) — the allocator Box/Vec use
- [Linked-list allocator](concepts/linked-list-allocator.md) — free list with first fit
- [alloc crate](concepts/alloc-crate.md) — Box, Vec, String without an OS
- [Interrupt-safe locking](concepts/interrupt-safe-lock.md) — why the heap lock disables interrupts

## Interrupts
- [Interrupt](concepts/interrupt.md) — "stop everything, run this now!"
- [Exception](concepts/exception.md) — an interrupt caused by your own code
- [IDT](concepts/idt.md) — the table of interrupt → handler
- [Divide by zero](concepts/divide-by-zero.md) — the example exception
- [PIC 8259](concepts/pic.md) — the interrupt mailbox hardware
- [IRQ](concepts/irq.md) — a numbered hardware interrupt line
- [IRQ remapping](concepts/irq-remapping.md) — moving IRQs to vectors 32+
- [EOI](concepts/eoi.md) — "done, send me the next interrupt"
- [Vector / offset](concepts/vector.md) — interrupt number
- [Triple fault](concepts/triple-fault.md) — when the CPU gives up and resets
- [sti / cli](concepts/sti-cli.md) — enable/disable interrupts
- [Masking interrupts](concepts/masked-irqs.md) — turning specific IRQs off

## Input
- [PS/2 keyboard](concepts/ps2-keyboard.md) — the old keyboard standard
- [Scancode](concepts/scancode.md) — the number a key press sends
- [Scancode set 1](concepts/scancode-set1.md) — the PC/AT layout we decode
- [Port I/O (0x60)](concepts/port-io.md) — talking to hardware via ports
- [Make / break codes](concepts/make-break.md) — press vs release

## Debugging & Logging
- [Serial port](concepts/serial-port.md) — the kernel's log wire
- [UART 16550](concepts/uart-16550.md) — the chip behind it
- [COM1 (0x3F8)](concepts/com1.md) — first serial port, shown by make run
- [Baud divisor / DLAB](concepts/baud-divisor.md) — picking the speed
- [Loopback test](concepts/loopback-test.md) — is a UART really there?
- [Emergency printing](concepts/force-unlock.md) — logging from a crash
- [addr2line](concepts/addr2line.md) — crash address to function name

## Display
- [VGA](concepts/vga.md) — legacy display hardware
- [VGA text buffer (0xB8000)](concepts/vga-text.md) — writing letters to screen
- [Attribute byte](concepts/attribute-byte.md) — color per character
- [VBE / VESA](concepts/vbe.md) — the BIOS graphics standard
- [Mode 0x112](concepts/vbe-mode-112.md) — 640x480x24bpp graphics mode
- [Linear framebuffer (LFB)](concepts/lfb.md) — raw pixel memory
- [Framebuffer driver](concepts/framebuffer.md) — our pixel writer (src/fb.rs)
- [Pitch](concepts/pitch.md) — bytes per screen row
- [Bitmap font](concepts/bitmap-font.md) — letters made of pixels
- [PSF font format](concepts/psf.md) — how Linux console fonts are stored

## Rust & Kernel
- [Freestanding kernel](concepts/freestanding.md) — Rust without an OS under it
- [no_std / no_main](concepts/no-std.md) — two magic attributes
- [_start](concepts/_start.md) — the kernel entry point
- [.bss section](concepts/bss.md) — zeroed statics, cleared by _start
- [panic_handler](concepts/panic-handler.md) — what happens on kernel panic
- [spin crate](concepts/spin.md) — locks without an OS
- [x86_64 crate](concepts/x86-64-crate.md) — safe wrappers over CPU instructions
- [pic8259 crate](concepts/pic8259-crate.md) — the PIC driver we use
- [Assembly (asm!)](concepts/asm.md) — raw CPU instructions in Rust
- [Release mode](concepts/release-mode.md) — why debug builds don't boot

## Build
- [Makefile](concepts/makefile.md) — automates build/run/clean
- [Kernel sectors](concepts/kernel-sectors.md) — Makefile passes the count
  to stage 2 automatically
