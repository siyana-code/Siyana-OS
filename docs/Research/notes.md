# Siyana OS — Teaching Notes

Each note explains **what** a component is, **why** it exists, and **how** it works
in Siyana OS. Read them in order; each one builds on the last.

---

## 1. Booting: BIOS → Stage 1 → Stage 2 → Kernel

### What is it?
When the CPU powers on, it has no operating system. It executes code from a
firmware called the **BIOS** (Basic Input/Output System), which looks for a boot
device and loads the first sector of it — 512 bytes — into memory at address
`0x7C00`, then jumps into it. This is the **boot sector** or Stage 1.

### Why do we need it?
A 512-byte sector can't hold an operating system. So Stage 1's only real job is
to load a bigger **Stage 2** loader from disk. Stage 2 does the heavy lifting:
setting up GDT, page tables, and entering modern CPU modes.

### How it works here
- `boot/boot.asm` — 512-byte stage 1. Prints a message, uses BIOS int `0x13`
  disk reads to load `boot/stage2.bin` into memory, jumps to it.
- `boot/stage2.asm` — loads `boot/kernel.bin` from disk to `0x10000`, builds a
  GDT, switches CPU to **protected mode** (32-bit), builds **page tables**,
  enables **long mode** (64-bit), and jumps into the Rust kernel entry point.

### Key idea
**Incremental booting** — each stage is small, does one job, and loads a bigger
next stage. There is no "big jump" from BIOS to a kernel.

---

## 2. CPU Modes: Real → Protected → Long Mode

### What is it?
x86 CPUs have historical execution modes:

| Mode        | Bits | Memory model                    |
|-------------|------|---------------------------------|
| Real mode   | 16   | 1 MB, segmented, no protection  |
| Protected   | 32   | 4 GB, segments, privilege rings |
| Long mode   | 64   | >4 GB, paging required          |

### Why do we need it?
We run 64-bit Rust code today — the hardware starts in 16-bit real mode for
backwards compatibility with 1980s PCs. We must walk the CPU through each mode
before jumping to the kernel.

---

## 3. GDT (Global Descriptor Table)

### What is it?
A table that defines **segments**: base, limit, and access rights for code and
data. In 64-bit long mode segmentation is mostly disabled, but the CPU still
requires a valid GDT to enter protected/long mode.

### Why?
To legally switch to protected mode. Also defines privilege rings (Ring 0 for
kernel, Ring 3 for userspace later).

---

## 4. Paging and Page Tables (Long Mode)

### What is it?
Paging translates **virtual addresses** (what a program uses) to **physical
addresses** (where RAM actually is). The CPU walks a 4-level table structure:
PML4 → PDPT → PD → PT.

### Why?
- Every process gets its own virtual address space (isolation later).
- We can mark pages present/not-present, writable, executable.
- Required to enter 64-bit long mode.

---

## 5. VGA Text Mode (`0xB8000`)

### What is it?
The first 32 KB (`0xB8000`–`0xBFFFF`) of memory is mapped to the legacy VGA
text buffer. Each character takes 2 bytes: ASCII byte + attribute byte
(foreground/background color).

### Why?
Simplest way to display output without any driver or framebuffer setup.

### Later
Replaced by a proper VGA text driver with scrolling, clearing, cursor, and
later a framebuffer driver.

---

## 6. Interrupts and IDT

### What is it?
An **interrupt** is an event that pauses the CPU and runs a handler. Examples:
- CPU **exceptions** (divide by zero, page fault) — synchronous, caused by the
  running instruction.
- **Hardware interrupts** (keyboard, timer) — asynchronous, from devices.

The **IDT (Interrupt Descriptor Table)** maps interrupt numbers (0–255) to
handler function addresses.

### Why?
Without exceptions the CPU triple-faults and resets. Without hardware interrupts
the kernel can never know a key was pressed or a timer ticked — only polling,
which wastes CPU.

### In Siyana OS
- `src/interrupts.rs` builds the IDT, registers `divide_by_zero_handler` and
  `keyboard_interrupt_handler`, and loads it with `lidt`.
- Handlers are `extern "x86-interrupt"` Rust functions — the `x86_64` crate
  generates correct entry/exit stubs.

---

## 7. PIC 8259 (Programmable Interrupt Controller)

### What is it?
Legacy hardware that collects device interrupts (IRQ0 timer, IRQ1 keyboard,
...) and forwards them to the CPU as interrupt numbers. Two PICs are chained:
IRQ0–7 on PIC1, IRQ8–15 on PIC2.

**Problem:** by default the IRQs map to interrupt numbers 0–15, which **collide
with CPU exception vectors** (0 = divide-by-zero, 14 = page fault...).

### Fix: remap
We reprogram the PICs so PIC1 IRQs go to vectors **32–39**, PIC2 to **40–47**.
`src/interrupts.rs` does this with `ChainedPics::new(32, 40)` + `initialize()`.

### Why do we send EOI?
After handling, the handler must write **End-Of-Interrupt** to the PIC,
otherwise that IRQ (and all same/lower priority) is never delivered again.
See `notify_end_of_interrupt` in the keyboard handler.

---

## 8. Keyboard Input via IRQ1

### What is it?
Pressing a key makes the keyboard controller (8042) assert IRQ1. The CPU then
runs our handler. The actual key data (a **scancode**) sits in the controller's
output buffer, readable from I/O port `0x60`.

### Why read port 0x60?
If the kernel never reads the scancode, the buffer stays full and **no more
IRQs arrive** — the handler fires once and then goes silent. Draining the port
(`Port::<u8>::new(0x60).read()`) acknowledges the key and re-arms IRQ1.

### Later
Scancodes will be translated to ASCII via a set-1/set-2 lookup table into a
proper input subsystem.

---

## 8.1 Memory Discovery (E820) & Bump Allocation

### What is it?
BIOS provides the physical memory map via E820. Stage 2 stores it at 0x7E00;
`src/memory.rs` sums usable regions (~127 MB on QEMU) and `alloc_pages` bumps a
pointer starting at 1 MiB.

### Why?
The kernel cannot use heap-style allocation until it knows where RAM is, and a
frame allocator is the foundation for a heap, page tables, and process isolation.

### Later
A real free-list allocator keyed to the E820 usable regions.

---

## 9. Freestanding Rust Kernel

### What is it?
`#![no_std]` + `#![no_main]` Rust binary: no standard library, no runtime, no
OS underneath. `_start` is our entry point; the linker script `linker.ld`
places the kernel at physical address `0x10000`.

### Why?
We **are** the OS — there is no libc, no allocator, no threading underneath us.
Everything must be built from `core` + hardware primitives.

---

## 10. PIT Timer (IRQ0)

### What is it?
The 8253/8254 PIT generates periodic interrupts on IRQ0. We program channel 0,
mode 3, divisor 11932 → 100 Hz.

### Why?
A kernel without time can never implement `sleep`, timeouts, or a preemptive
scheduler. IRQ0 gives us a heartbeat. Our handler bumps `TICKS` and, once per
second, refreshes the green on-screen counter.

### Order of operations
IDT handler registered → PICs initialized (IRQ0+IRQ1 unmasked) → PIT programmed
→ `sti` enables interrupts.

---

## 11. VESA Framebuffer (GUI groundwork)

### What is it?
VGA text mode puts ASCII into a character buffer. A **VESA (VBE) linear
framebuffer** gives us a raw array of pixels — the base of every modern GUI.

### Why?
A GUI needs per-pixel graphics: icons, windows, fonts, wallpapers. Text mode
can't draw shapes or blend colors.

### How it works here
- `boot/stage2.asm` calls BIOS `int 0x10`, function `0x4F01` (get VBE mode
  info for mode `0x112` = 640×480×24bpp), reads the linear framebuffer address
  out of the ModeInfoBlock, stores it (with pitch/width/height) at physical
  `0x7C00`, then sets the mode via function `0x4F02`.
- Page tables now identity-map 0–4 GB (four 1 GB pages) so the kernel can
  write to the LFB no matter where the chipset placed it.
- `src/fb.rs` reads the info block and plots pixels; `draw_test_pattern()`
  paints a gradient as proof.

### Next
Bitmap font rendering, double buffering, then a window manager.

---

## 12. Build System (Makefile + raw disk image)

### What is it?
The Makefile assembles both boot stages with NASM, builds the kernel in release
mode, converts the ELF to a raw binary with `objcopy`, pads it to whole 512-byte
sectors, and concatenates everything into `boot/disk.img`.

### Why the sector-count check?
Stage 2 loads the kernel from disk by reading N sectors. If the kernel grows
and N isn't updated, the kernel loads truncated and crashes mysteriously. The
Makefile now computes the needed N from the binary size and fails the build if
`stage2.asm` disagrees.

### Why release mode?
Debug builds include stack-protector and overflow checks that need symbols a
freestanding kernel doesn't have.
