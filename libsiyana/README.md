# libsiyana

The Siyana OS userspace API. Everything an app needs from the OS goes
through here: console, files, time, processes, events and IPC.

| Backend | When | Status |
|---|---|---|
| Linux | `target_os = "linux"` | Works today, built on `std` + `libc` |
| Siyana | `target_os = "none"` | Native system calls ([docs/abi.md](../docs/abi.md)); compiles today, runs once the kernel implements them (Phase 4) |

Write your app once against this API; it builds and runs on Ubuntu now, and
recompiles for Siyana unchanged.

## Use it

```toml
[dependencies]
libsiyana = { git = "https://github.com/siyana-code/Siyana-OS", branch = "develop" }
```

```rust
#![cfg_attr(not(target_os = "linux"), no_std, no_main)]
use libsiyana::prelude::*;

libsiyana::entry!(app_main);

fn app_main() -> i32 {
    println!("hello from pid {}", libsiyana::process::id());
    0 // exit code
}
```

## The one rule: `core` + `alloc` + `libsiyana`, never `std`

`std` does not exist on Siyana (yet). If your app compiles for
`x86_64-unknown-none`, it will run on Siyana:

```bash
rustup target add x86_64-unknown-none   # once
cargo check --target x86_64-unknown-none
```

Put that in your CI. `libsiyana::prelude::*` gives you `String`, `Vec`,
`Box`, `format!`, `vec!`, `ToString`, `Duration` and the print macros.
Collections like `BTreeMap` come from `alloc::collections`. Crates work if
they support `no_std` + `alloc` (e.g. `serde` with
`default-features = false, features = ["alloc", "derive"]`).

## API at a glance

| Module | What |
|---|---|
| `io` | `print!`/`println!`/`eprintln!`, `read_line`, raw `read`/`write` on handles |
| `fs` | `File`, `OpenOptions`, `read`, `read_to_string`, `write`, `create_dir_all`, `remove_*`, `read_dir`, `exists` |
| `time` | `Instant`, `sleep`, `unix_time` |
| `env` | `args()`, `var()`, `vars()` |
| `process` | `Command` (args, env, cwd, stdio, process group), `kill`/`kill_group`, `wait`/`try_wait`/`try_wait_any`/`reap`, `become_reaper`, `exit`, `id` |
| `event` | `wait(timeout, handles)` → `Shutdown` / `ChildExited` / `Readable(h)` |
| `ipc` | `Listener::bind(name)` / `accept`, `Stream::connect(name)`, `shutdown_write` |

## Events, not signals

Siyana has no signal handlers. Shutdown requests and child exits arrive as
events you read in your main loop; see `examples/supervisor.rs` (a
service manager in ~150 lines) and [docs/abi.md §6](../docs/abi.md).

| Linux habit | libsiyana |
|---|---|
| `sigaction(SIGTERM, ...)` | `Event::Shutdown` from `event::wait` |
| `SIGCHLD` handler + `waitpid(-1, WNOHANG)` | `Event::ChildExited` + `process::reap()` |
| `poll()` on a socket | `event::wait(timeout, &[listener.handle()])` |
| `prctl(PR_SET_CHILD_SUBREAPER)` | `process::become_reaper()` |
| Unix socket at a path | `ipc::Listener::bind("name")` |

On Linux, IPC names become Unix sockets under `$XDG_RUNTIME_DIR/siyana-ipc/`
(override with `SIYANA_IPC_DIR`; keep it short: socket paths are limited
to 108 bytes).

## Develop

```bash
cargo test                                          # Linux backend tests
cargo run --example hello -- a b
cargo run --example supervisor                      # then, in another terminal:
cargo run --example supervisor -- status
cargo check --target x86_64-unknown-none --examples # Siyana build
```
