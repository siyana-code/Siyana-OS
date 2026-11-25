# Siyana ABI — version 0 (draft)

The contract between the Siyana kernel and userspace programs. The numbers
and structs live in [`abi/src/lib.rs`](../abi/src/lib.rs) (crate
`siyana-abi`), shared by the kernel and [`libsiyana`](../libsiyana). This
file is the prose; if the two disagree, fix one of them.

Apps should not make system calls directly: use `libsiyana`.

Status: **draft**. Nothing here is implemented in the kernel yet (Phase 4).
Until v1 is declared, numbers may change; `libsiyana` will change with them,
so apps written against `libsiyana` keep compiling.

## 1. Executables

- ELF64, x86-64, statically linked, no interpreter (`PT_INTERP`).
- `ET_EXEC` or `ET_DYN` (position-independent). For `ET_DYN` the loader
  picks the base address and applies `R_X86_64_RELATIVE` relocations, the
  only kind allowed. Plain `cargo build --target x86_64-unknown-none` makes
  this, so apps need no special build flags.
- Entry: the ELF entry point (`_start`, provided by `libsiyana`). The kernel
  enters it with `rsp` at the top of a fresh stack; `_start` realigns it.
  Arguments and environment are fetched with the `args` / `env` calls, not
  from the stack.

## 2. Calling convention

| | |
|---|---|
| Instruction | `syscall` |
| Call number | `rax` |
| Arguments | `rdi`, `rsi`, `rdx`, `r10`, `r8`, `r9` |
| Result | `rax`: `>= 0` success, negative = `-(error code)` |
| Clobbered | `rcx`, `r11` (by the CPU); everything else preserved |

Strings are `(pointer, length)` pairs of UTF-8, **never** NUL-terminated.
The kernel validates every pointer and length: a bad one fails with
`INVALID_INPUT`, never a kernel crash.

## 3. Handles

A **handle** is a small `u32` naming a kernel object the process owns: a
file, directory, IPC listener or stream, or console. Handles 0, 1, 2 are
stdin, stdout, stderr. `close` releases a handle; all are released at exit.

## 4. System calls

| # | Name | Arguments | Returns |
|---|---|---|---|
| 0 | `exit` | `code: i32` | never returns |
| 1 | `write` | `handle, buf, len` | bytes written |
| 2 | `read` | `handle, buf, len` | bytes read; 0 = end of input |
| 3 | `close` | `handle` | 0 |
| 4 | `open` | `path, path_len, flags` | handle |
| 5 | `mkdir` | `path, path_len, flags` | 0 |
| 6 | `remove` | `path, path_len, flags` | 0 |
| 7 | `read_dir` | `handle, buf, len` | bytes of `DirRecord`s; 0 = done |
| 8 | `clock` | `clock` | nanoseconds |
| 9 | `sleep` | `nanoseconds` | 0 |
| 10 | `getpid` | — | pid |
| 11 | `spawn` | `*const SpawnArgs` | child pid |
| 12 | `kill` | `pid, how` | 0 |
| 13 | `wait` | `pid or 0 (any), flags, *mut ExitInfo` | pid; 0 = none finished (`NOHANG`) |
| 14 | `set_reaper` | — | 0 |
| 15 | `event_wait` | `timeout_ns, *const u32 watch, watch_len, *mut Event out, out_len` | events written |
| 16 | `ipc_listen` | `name, name_len` | listener handle |
| 17 | `ipc_accept` | `listener` | stream handle |
| 18 | `ipc_connect` | `name, name_len` | stream handle |
| 19 | `args` | `buf, len` | total bytes needed |
| 20 | `env` | `buf, len` | total bytes needed |
| 21 | `mem_map` | `len` | address of zeroed read/write pages |
| 22 | `mem_unmap` | `address, len` | 0 |
| 23 | `ipc_shutdown_write` | `stream` | 0 |

Flag values are in `siyana-abi` (`open::*`, `mkdir::*`, `remove::*`,
`clock::*`, `kill::*`, `wait::*`, `spawn::*`).

### Notes

- **`open`** flags: `READ`, `WRITE`, `CREATE`, `TRUNCATE`, `APPEND`,
  `DIRECTORY` (open a directory for `read_dir`).
- **`read_dir`** fills `buf` with records: a 4-byte `DirRecord` header
  (`name_len: u16`, `kind: u8`, reserved `u8`) followed by `name_len` bytes
  of name. `.` and `..` are not listed.
- **`args` / `env`**: entries each followed by one 0 byte (env entries are
  `KEY=value`). If `len` is smaller than the total, nothing is copied and
  the total is returned: call with `len = 0` first to size the buffer.
- **`spawn`**: the child gets exactly the argv, environment, working
  directory and stdio handles in `SpawnArgs`. `handle::INHERIT` shares the
  parent's stream, `handle::NULL` gives nothing. `spawn::NEW_GROUP` makes
  the child lead a new process group.
- **`wait`** fills `ExitInfo { kind, code }`: `CODE` (called `exit`),
  `KILLED`, or `CRASHED` (killed by the kernel after a fault).
- **`mem_map`** may back pages lazily (on first touch).

## 5. Processes, groups and reaping

Every process has a parent and belongs to a process group. `kill` with the
`GROUP` flag targets the whole group. When a process exits, its children
are re-parented to the nearest ancestor that called `set_reaper` (init, at
the top); the parent collects exit status with `wait`.

## 6. Events instead of signals

Siyana has **no asynchronous signal handlers**. Each process has an event
queue, read with `event_wait`:

| Event | Meaning |
|---|---|
| `SHUTDOWN` | Someone sent `kill(pid, TERMINATE)`, or Ctrl-C on its terminal |
| `CHILD_EXITED` | At least one child finished; collect with `wait` |
| `READABLE(handle)` | A watched handle has data, or a listener has a client |

`event_wait` blocks until an event is pending, a watched handle is
readable, or the timeout passes (`u64::MAX` = forever, 0 = poll).

`TERMINATE` turns into a `SHUTDOWN` event only for a process that has
called `event_wait` at least once (it has shown it handles events). Any
other process is ended by `TERMINATE` (`ExitInfo.kind = KILLED`). `KILL`
always ends the target immediately.

Why: signal handlers run at arbitrary points and may only call a tiny set
of "async-signal-safe" functions, a constant source of bugs. A queue read
from the main loop has neither problem.

## 7. IPC

Named, connection-oriented byte streams between processes on one machine.
`ipc_listen(name)` registers a name (1–64 characters of `A-Z a-z 0-9 . _ -`);
a second live listener for the same name gets `ALREADY_EXISTS`. Closing the
listener handle unregisters it. `ipc_connect(name)` returns a stream to a
listener, or `NOT_FOUND`. Streams are full duplex; `ipc_shutdown_write`
signals end of input to the peer while still allowing reads.

## 8. Error codes

| Code | Name | Code | Name |
|---|---|---|---|
| 1 | `NOT_FOUND` | 9 | `OUT_OF_MEMORY` |
| 2 | `PERMISSION_DENIED` | 10 | `BAD_HANDLE` |
| 3 | `ALREADY_EXISTS` | 11 | `NO_CHILD` |
| 4 | `INVALID_INPUT` | 12 | `UNSUPPORTED` (call not implemented) |
| 5 | `WOULD_BLOCK` | 13 | `BROKEN_PIPE` |
| 6 | `INTERRUPTED` | 14 | `TIMED_OUT` |
| 7 | `NOT_A_DIRECTORY` | 15 | `NOT_EMPTY` |
| 8 | `IS_A_DIRECTORY` | 255 | `OTHER` |

## 9. Not yet in v0

Threads, shared memory, users and permissions, networking, graphics and
input, time zones, file metadata (size, timestamps, permissions), seeking,
pipes between processes. Each will be added here before the kernel
implements it.
