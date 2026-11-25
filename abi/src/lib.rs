//! Siyana OS ABI, version 0: the contract between the kernel and userspace.
//!
//! Shared by the kernel (which implements the calls) and `libsiyana` (which
//! makes them), so the two can never disagree on a number. The prose spec is
//! `docs/abi.md`; keep the two in sync.
//!
//! # Calling convention
//!
//! `syscall` instruction. `rax` = call number, arguments in `rdi`, `rsi`,
//! `rdx`, `r10`, `r8`, `r9`. The result comes back in `rax`: a value `>= 0`
//! is success, a negative value is `-(error code)` from [`error`]. The CPU
//! clobbers `rcx` and `r11`; every other register is preserved.
//!
//! Strings are passed as (pointer, length) pairs of UTF-8, never
//! NUL-terminated.

#![no_std]

/// ABI version. Bumped on any incompatible change.
pub const VERSION: u32 = 0;

/// System-call numbers.
pub mod nr {
    /// `exit(code: i32) -> !`
    pub const EXIT: usize = 0;
    /// `write(handle, buf, len) -> bytes written`
    pub const WRITE: usize = 1;
    /// `read(handle, buf, len) -> bytes read` (0 = end of file / peer closed)
    pub const READ: usize = 2;
    /// `close(handle) -> 0`
    pub const CLOSE: usize = 3;
    /// `open(path, path_len, flags: open::*) -> handle`
    pub const OPEN: usize = 4;
    /// `mkdir(path, path_len, flags: mkdir::*) -> 0`
    pub const MKDIR: usize = 5;
    /// `remove(path, path_len, flags: remove::*) -> 0`
    pub const REMOVE: usize = 6;
    /// `read_dir(handle, buf, len) -> bytes`: packed [`DirRecord`](super::DirRecord)s
    /// from a handle opened with `open::DIRECTORY`. 0 = no more entries.
    pub const READ_DIR: usize = 7;
    /// `clock(clock: clock::*) -> nanoseconds`
    pub const CLOCK: usize = 8;
    /// `sleep(nanoseconds) -> 0`
    pub const SLEEP: usize = 9;
    /// `getpid() -> pid`
    pub const GETPID: usize = 10;
    /// `spawn(args: *const SpawnArgs) -> pid`
    pub const SPAWN: usize = 11;
    /// `kill(pid, how: kill::*) -> 0`. Terminate delivers an
    /// `event::SHUTDOWN` to the target; Kill ends it immediately.
    pub const KILL: usize = 12;
    /// `wait(pid or wait::ANY, flags: wait::*, status: *mut ExitInfo) -> pid`.
    /// With `wait::NOHANG` and nothing finished, returns 0.
    pub const WAIT: usize = 13;
    /// `set_reaper() -> 0`: orphaned descendants are re-parented to the caller.
    pub const SET_REAPER: usize = 14;
    /// `event_wait(timeout_ns, watch: *const u32, watch_len, out: *mut Event, out_len) -> count`.
    /// Blocks until an event is pending or a watched handle is readable.
    /// `timeout_ns == u64::MAX` waits forever; 0 polls.
    pub const EVENT_WAIT: usize = 15;
    /// `ipc_listen(name, name_len) -> listener handle`
    pub const IPC_LISTEN: usize = 16;
    /// `ipc_accept(listener) -> stream handle`
    pub const IPC_ACCEPT: usize = 17;
    /// `ipc_connect(name, name_len) -> stream handle`
    pub const IPC_CONNECT: usize = 18;
    /// `args(buf, len) -> total bytes`: arguments, each followed by a 0 byte.
    /// If `len` is too small nothing is copied; call again with the total.
    pub const ARGS: usize = 19;
    /// `env(buf, len) -> total bytes`: `KEY=value` entries, each followed by
    /// a 0 byte. Same sizing rule as `ARGS`.
    pub const ENV: usize = 20;
    /// `mem_map(len) -> address`: fresh zeroed, read/write pages.
    pub const MEM_MAP: usize = 21;
    /// `mem_unmap(address, len) -> 0`
    pub const MEM_UNMAP: usize = 22;
    /// `ipc_shutdown_write(stream) -> 0`: stop sending; the peer's reads
    /// return 0 once it has drained what was sent.
    pub const IPC_SHUTDOWN_WRITE: usize = 23;
}

/// Error codes. Syscalls return the negated value.
pub mod error {
    pub const NOT_FOUND: isize = 1;
    pub const PERMISSION_DENIED: isize = 2;
    pub const ALREADY_EXISTS: isize = 3;
    pub const INVALID_INPUT: isize = 4;
    pub const WOULD_BLOCK: isize = 5;
    pub const INTERRUPTED: isize = 6;
    pub const NOT_A_DIRECTORY: isize = 7;
    pub const IS_A_DIRECTORY: isize = 8;
    pub const OUT_OF_MEMORY: isize = 9;
    pub const BAD_HANDLE: isize = 10;
    pub const NO_CHILD: isize = 11;
    /// The kernel does not implement this call (yet).
    pub const UNSUPPORTED: isize = 12;
    pub const BROKEN_PIPE: isize = 13;
    pub const TIMED_OUT: isize = 14;
    pub const NOT_EMPTY: isize = 15;
    pub const OTHER: isize = 255;
}

/// Handles every process starts with.
pub mod handle {
    pub const STDIN: u32 = 0;
    pub const STDOUT: u32 = 1;
    pub const STDERR: u32 = 2;
    /// In `SpawnArgs`: give the child nothing on this stream.
    pub const NULL: u32 = u32::MAX;
    /// In `SpawnArgs`: the child shares the parent's stream.
    pub const INHERIT: u32 = u32::MAX - 1;
}

pub mod open {
    pub const READ: usize = 1 << 0;
    pub const WRITE: usize = 1 << 1;
    pub const CREATE: usize = 1 << 2;
    pub const TRUNCATE: usize = 1 << 3;
    pub const APPEND: usize = 1 << 4;
    /// Open a directory for `READ_DIR`.
    pub const DIRECTORY: usize = 1 << 5;
}

pub mod mkdir {
    /// Create missing parents too; an existing directory is not an error.
    pub const PARENTS: usize = 1 << 0;
}

pub mod remove {
    /// Remove a directory (it must be empty unless `RECURSIVE`).
    pub const DIRECTORY: usize = 1 << 0;
    pub const RECURSIVE: usize = 1 << 1;
}

pub mod clock {
    /// Nanoseconds since boot; never goes backwards.
    pub const MONOTONIC: usize = 0;
    /// Nanoseconds since 1970-01-01 UTC.
    pub const REALTIME: usize = 1;
}

pub mod kill {
    /// Ask the process to stop: it receives `event::SHUTDOWN`.
    pub const TERMINATE: usize = 1;
    /// Stop it now. Cannot be caught.
    pub const KILL: usize = 2;
    /// Flag: apply to the target's whole process group.
    pub const GROUP: usize = 1 << 8;
}

pub mod wait {
    /// `pid` value meaning "any child".
    pub const ANY: usize = 0;
    /// Return 0 instead of blocking when no child has finished.
    pub const NOHANG: usize = 1 << 0;
}

pub mod spawn {
    /// Put the child in a new process group led by itself.
    pub const NEW_GROUP: u32 = 1 << 0;
}

/// Event kinds delivered by `EVENT_WAIT`.
pub mod event {
    /// Someone asked this process to stop (`kill::TERMINATE`).
    pub const SHUTDOWN: u32 = 1;
    /// A child process finished; collect it with `WAIT`.
    pub const CHILD_EXITED: u32 = 2;
    /// A watched handle has data (or a connection) ready.
    pub const READABLE: u32 = 3;
}

/// A (pointer, length) string in a syscall argument struct.
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct Str {
    pub ptr: *const u8,
    pub len: usize,
}

/// Argument block for `SPAWN`.
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct SpawnArgs {
    /// Program to run.
    pub path: Str,
    /// Full argument list, including argv[0].
    pub argv: *const Str,
    pub argc: usize,
    /// `KEY=value` entries. The child gets exactly these.
    pub envp: *const Str,
    pub envc: usize,
    /// Working directory; `len == 0` means inherit the parent's.
    pub cwd: Str,
    /// Handles for the child's stdin/stdout/stderr, or `handle::NULL` /
    /// `handle::INHERIT`.
    pub stdio: [u32; 3],
    /// `spawn::*` flags.
    pub flags: u32,
}

/// How a process ended, filled in by `WAIT`.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ExitInfo {
    /// One of `exit_kind::*`.
    pub kind: u32,
    /// Exit code for `exit_kind::CODE`, otherwise 0.
    pub code: i32,
}

pub mod exit_kind {
    /// Called `exit(code)`.
    pub const CODE: u32 = 0;
    /// Stopped by `kill::KILL`, or by `kill::TERMINATE` without exiting itself.
    pub const KILLED: u32 = 1;
    /// Killed by the kernel after a fault (page fault, divide by zero...).
    pub const CRASHED: u32 = 2;
}

/// One entry from `EVENT_WAIT`.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Event {
    /// One of `event::*`.
    pub kind: u32,
    /// For `READABLE`: the handle. Otherwise 0.
    pub handle: u32,
}

/// Header of one directory entry from `READ_DIR`; `name_len` bytes of UTF-8
/// name follow it immediately, then the next record.
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct DirRecord {
    pub name_len: u16,
    /// One of `dir_kind::*`.
    pub kind: u8,
    pub _reserved: u8,
}

pub mod dir_kind {
    pub const FILE: u8 = 0;
    pub const DIRECTORY: u8 = 1;
    pub const OTHER: u8 = 2;
}
