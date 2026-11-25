# Siyana OS — Applications Roadmap (userspace)

Owner: **Kaushalya**. Kernel-side work is tracked in [plan.md](plan.md).

## ABI decision: Option A (native Siyana system calls)

Siyana has its own system-call interface, like Redox OS, not Linux's. Consequences:

- Apps talk to the OS through **`libsiyana`** (Rust crate, kernel side provides
  it) and later through Siyana's own libc and a Rust `std` target.
- Until those exist, write apps against `libsiyana`'s API with its **Linux
  backend**, so they build and run on Ubuntu today. When the Siyana backend
  lands, the same app recompiles for Siyana.
- Existing programs (rustc, gcc, git, Lua, ...) must be **ported**: they need
  the Siyana libc / Rust std target first (kernel side), then the port itself
  (app side).

## Dependency key

| Key | Meaning |
|---|---|
| **now** | Can be built today on Linux against `libsiyana` |
| **proc** | Needs processes + syscalls on Siyana (Phase 4) |
| **fs** | Needs the filesystem service (Phase 5) |
| **net** | Needs network stack (kernel side) |
| **libc** | Needs Siyana libc / Rust std target (kernel side) |
| **gui** | Needs display server protocol (Phase 6.5) |
| **audio** | Needs sound driver (kernel side) |

## Tier 0: System services

| # | App | Purpose | Study | Needs |
|---|---|---|---|---|
| 0.1 | **init / service manager** 🔧 | First userspace process; starts and supervises services. v0.1 runs on Linux: [Siyana-Service-Manager](https://github.com/siyana-code/Siyana-Service-Manager) | Redox `init`, systemd concepts | proc |
| 0.2 | **Logging daemon** | System logs (like `syslog`/`journald`); collects kernel log too | — | proc |

## Tier 1: Minimal usable system (CLI)

| # | App | Notes / study | Needs |
|---|---|---|---|
| 1.1 | **Shell** | Pipes, redirection, env vars, scripting. Study `nushell`, Redox `ion` | now → proc |
| 1.2 | **Core utilities** | `ls`, `cp`, `mv`, `rm`, `mkdir`, `cat`, `echo`, `ps`, `kill`, `mount`, `chmod`, `find`, `grep`, `head`/`tail`, `wc`, `df`, `free`, `uname`, `date`. Write your own, or port `uutils/coreutils` | now → fs, libc for the port |
| 1.3 | **Text editor (terminal)** | `nano`-style first, modal later. Study `helix`, `kibi` | now → fs |
| 1.4 | **Pager** | `less` / `more` | now |
| 1.5 | **User management** | `login`, `passwd`, `useradd`, `su`/`sudo` | proc, fs (+ kernel users/permissions) |
| 1.6 | **Process/system monitor** | `top`-like. Study `bottom` | proc |
| 1.7 | **Network tools** | `ping`, `ifconfig`/`ip`, `curl`/`wget`-like downloader, `nslookup` | net |
| 1.8 | **Archive tools** | tar, gzip, zip (crates: `tar`, `flate2`, `zip`) | now → fs |

## Tier 2: Developer essentials

| # | App | Notes / study | Needs |
|---|---|---|---|
| 2.1 | **Package manager** | Install, remove, update, resolve dependencies. **Design early**: package format + repository layout | now (design) → fs, net |
| 2.2 | **Rust toolchain port** | `rustc` + `cargo` running natively; self-hosting milestone | libc, fs |
| 2.3 | **C toolchain port** | GCC/Clang + `make` | libc, fs |
| 2.4 | **Git** | Port git, or use `gitoxide` (pure Rust) | libc, fs, net |
| 2.5 | **Debugger (userspace)** | GDB port or your own, using the kernel's debug syscalls | libc, proc |
| 2.6 | **Build tools** | make, CMake | libc, fs |
| 2.7 | **SSH client/server** | Remote access (`russh` crate) | net |
| 2.8 | **Scripting language** | Lua port first (easier), Python later | libc |
| 2.9 | **Terminal multiplexer** | `tmux`-like. Study `zellij` | proc |
| 2.10 | **Man/help system** | Built-in documentation | now → fs |

## Tier 3: Graphical environment

| # | App | Notes / study | Needs |
|---|---|---|---|
| 3.1 | **GUI toolkit** | Shared look for all apps. Study `iced`, `slint`, `egui` | now (prototype on Linux) → gui |
| 3.2 | **Desktop shell** | Taskbar, app launcher, notifications, system tray | gui |
| 3.3 | **Window manager policy** | Placement, tiling, focus rules (the compositor itself is kernel-side) | gui |
| 3.4 | **Terminal emulator** | GPU-accelerated later. Study `alacritty`, `wezterm` | gui |
| 3.5 | **File manager** | | gui, fs |
| 3.6 | **Code editor / IDE (GUI)** | Syntax highlighting, LSP. Study `lapce`, `zed` | gui, fs |
| 3.7 | **Settings app** | Display, network, users, updates, themes | gui |
| 3.8 | **Software center** | GUI front end for the package manager (2.1) | gui, 2.1 |
| 3.9 | **Font rendering + fonts** | `fontdue`, `cosmic-text` | now (library) |

## Tier 4: Everyday apps

| # | App | Notes | Needs |
|---|---|---|---|
| 4.1 | **Web browser** | Hardest app by far; port an engine (Servo is Rust) | libc, gui, net |
| 4.2 | **Image viewer** | | gui, fs |
| 4.3 | **PDF viewer** | | gui, fs |
| 4.4 | **Audio/video player** | Needs codec ports | gui, audio |
| 4.5 | **Calculator** | Good first GUI app | now → gui |
| 4.6 | **Screenshot tool** | | gui |
| 4.7 | **Clipboard manager** | | gui |
| 4.8 | **Network manager GUI** | Wi-Fi, Ethernet, VPN | gui, net |
| 4.9 | **Disk utility / partition manager** | | gui, fs |
| 4.10 | **Task manager GUI** | | gui, proc |
| 4.11 | **Archive manager GUI** | Front end for 1.8 | gui, 1.8 |
| 4.12 | **Notes / text app** | | gui, fs |

## Tier 5: Developer-OS differentiators

| # | App | Notes | Needs |
|---|---|---|---|
| 5.1 | **Container/sandbox CLI** | User-facing tool over the kernel's isolation features | proc (+ kernel isolation) |
| 5.2 | **Tracer/profiler UI** | Front end for the kernel's tracing interface | proc (+ kernel tracing) |
| 5.3 | **API/HTTP client** | Built-in tool for testing APIs | net |
| 5.4 | **Local database** | SQLite port | libc, fs |
| 5.5 | **Dev environment manager** | Per-project toolchain versions | fs, 2.1 |

## Suggested start order

1. **1.1 Shell, 1.4 Pager, 1.3 Editor, 4.5 Calculator (CLI)**: build on Linux now.
2. **2.1 Package manager design**: package format + repo layout, on paper first.
3. **3.9 Fonts + 3.1 GUI toolkit prototype**: on Linux, so the look is ready when the display server lands.
4. Everything else as the kernel unlocks its "Needs".
