# Siyana OS — Production Project Plan

Target: an operating system with a stable kernel, memory management, multitasking,
drivers, a userspace, and a defined system-call interface — engineered, not a toy.

## Status legend
- ✅ done
- 🔧 in progress
- ⬜ planned

---

## Phase 1 — Foundation (current)

| Item | Status |
|------|--------|
| BIOS stage-1 bootloader | ✅ |
| Stage-2 loader: GDT, protected, long mode | ✅ |
| Rust freestanding kernel, VGA output | ✅ |
| IDT + divide-by-zero handler | ✅ |
| PIC remap + keyboard IRQ1 handler | 🔧 (scancode drained; decoding pending) |
| Proper VGA text driver (scroll, clear, cursor) | ⬜ |
| Scancode → keycode → ASCII translation | ⬜ |
| Timer interrupt (IRQ0) + tick counter | ✅ |

## Phase 2 — Memory

| Item | Notes |
|------|-------|
| Physical frame allocator (E820 + bump) | ✅ |
| LBA kernel loader (INT 13h AH=42h, up to 448 KiB) | ✅ |
| Heap allocator (4 MiB, linked_list_allocator) | ✅ |
| Virtual memory management | map/unmap API over page-table levels |
| Page fault handler (CR2 + error code decoded) | ✅ |

## Phase 3 — Concurrency

| Item | Notes |
|------|-------|
| Preemptive scheduler | Timer IRQ driven context switch |
| Kernel tasks (threads) | per-task stack + saved register state |
| Synchronization | Mutex (exists), spinlocks, semaphores, wait queues |
| Deadlock hygiene | lock ordering rules documented |

## Phase 4 — Protection

| Item | Notes |
|------|-------|
| Ring 3 userspace | separate page tables per process |
| System call interface | `syscall`/`sysret` or int 0x80 gate |
| ELF loading of user programs | from disk |
| Syscall validation | never trust userspace pointers |

## Phase 5 — Storage & Devices

| Item | Notes |
|------|-------|
| AHCI/SATA or ATA PIO disk driver | read/write sectors |
| Simple filesystem | first a flat/tar-like, then FAT16 or custom |
| PS/2 keyboard driver | proper driver, not raw IRQ handler |
| Serial/UART logging (COM1, serial_println!) | ✅ |

## Phase 6 — Userspace

| Item | Notes |
|------|-------|
| `init` process | first userspace process, spawns others |
| Minimal libc for userspace | malloc, string, IO wrappers over syscalls |
| Shell | ls, cat, run, echo, help |
| ELF-based program loading | |

## Phase 6.5 — Graphics & GUI

| Item | Notes |
|------|-------|
| VBE/VESA linear framebuffer | mode set in stage 2, address passed to kernel |
| Framebuffer driver | pixel plotting, fill, lines |
| Bitmap font renderer | glyph rendering on framebuffer |
| Double buffering / back buffer | flicker-free drawing |
| Window manager core | rectangles, titles, focus, event routing |
| Widget toolkit | buttons, labels, text fields |
| Compositor | layered windows, animation |
| Mouse driver | PS/2 pointing device in GUI mode |
| Desktop shell | wallpaper, taskbar, app launcher |

## Phase 7 — Production hardening

| Item | Notes |
|------|-------|
| Kernel panic messages with register dump | 🔧 message + location on serial; register dump pending |
| Watchdog / triple-fault recovery path | |
| Testing: QEMU boot tests in CI | GitHub Actions build + QEMU smoke test |
| Fuzz/property tests for parsers & allocators | |
| Documentation of every subsystem (see docs/Research/notes.md style) | |
| Release process: tagged versions, `main` = release, `develop` = integration | |

## Platform work from the apps list (kernel side, Claude)

Userspace apps are owned by Kaushalya: see [apps.md](apps.md).
ABI decision: **Option A**, native Siyana system calls (Redox-style).

| Item | Unlocks (apps.md key) | Notes |
|------|------|-------|
| Siyana syscall ABI spec | all | ✅ v0 draft: [abi.md](abi.md), crate `abi/` |
| `libsiyana` crate (Linux + Siyana backends) | now | ✅ v0: io, fs, time, env, process, event, ipc. Linux backend tested; Siyana backend compiles |
| Process model: spawn, exit, wait, signals | proc | Phase 4 |
| Device manager | hardware | PCI enumeration, driver binding (study Redox `pcid`) |
| Filesystem service: VFS + FAT32, then ext2 | fs | `fatfs`, `ext2-rs`, RedoxFS |
| Network stack: NIC driver + TCP/IP, DHCP, DNS | net | `smoltcp` |
| Siyana libc + Rust `std` target | libc | Study `relibc`; unblocks every port (rustc, gcc, git, Lua) |
| Kernel log interface | logging daemon | Read the kernel log from userspace |
| Users, groups, permissions | user management | Needed by login/passwd/sudo |
| Display server / compositor + protocol | gui | Study Redox `orbital`, `smithay` |
| Input: PS/2 mouse, keyboard events to userspace | gui | |
| Sound driver (AC97 / Intel HDA) | audio | |
| Kernel GDB stub | debugger | Debug Siyana from Ubuntu over serial |
| Debug syscalls (ptrace-like) | debugger | For the userspace debugger (2.5) |
| Isolation primitives (namespaces/sandbox) | 5.1 containers | |
| Tracing/profiling interface | 5.2 tracer | eBPF-style observability |
| Linux compatibility layer | thousands of Linux tools | Optional, late: Linux syscalls on top of the native ABI |

## Development workflow (agreed process)

For every feature or phase:

1. **Build** — implement it, `make`, fix warnings/errors.
2. **Automated smoke test** — boot in QEMU (`-display none -d int`), confirm no
   triple fault; capture a screenshot and check pixels/text.
3. **Research note** — add/refresh notes under `docs/Research/` (every new
   keyword, concept, or mechanism gets its own file, written simply).
4. **Ask you to verify** — you run `make run` and confirm it works.
5. **Only then commit + push** to `develop`, with a descriptive message.

`main` is for production releases; merge from `develop` via PR.

## Branching model
- `main` — production releases; merge only via PR from `develop`
- `develop` — integration branch; **default**
- `feature/<name>` — branched from `develop`
- `release/<x.y>` — stabilization for a release
- `hotfix/<name>` — from `main`, back into `main` + `develop`
