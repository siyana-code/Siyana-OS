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
| Timer interrupt (IRQ0) + tick counter | ⬜ |

## Phase 2 — Memory

| Item | Notes |
|------|-------|
| Physical frame allocator | Bitmap or free-list over usable RAM (parse BIOS map) |
| Heap allocator | linked_list_allocator or bump → growing heap |
| Virtual memory management | map/unmap API over page-table levels |
| Page fault handler | Must diagnose R/W, present, user/supervisor bits |

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
| Serial/UART logging | for debugging and production diagnostics |

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
| Kernel panic messages with register dump | |
| Watchdog / triple-fault recovery path | |
| Testing: QEMU boot tests in CI | GitHub Actions build + QEMU smoke test |
| Fuzz/property tests for parsers & allocators | |
| Documentation of every subsystem (see docs/NOTES.md style) | |
| Release process: tagged versions, `main` = release, `develop` = integration | |

## Branching model
- `main` — production releases; merge only via PR from `develop`
- `develop` — integration branch; **default**
- `feature/<name>` — branched from `develop`
- `release/<x.y>` — stabilization for a release
- `hotfix/<name>` — from `main`, back into `main` + `develop`
