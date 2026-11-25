//! Linux backend: implements the Siyana ABI's behaviour with `std` + `libc`,
//! so apps can be developed and tested on Ubuntu before Siyana runs them.
//!
//! Handles are Linux file descriptors. Siyana's event queue is emulated with
//! signal handlers that only set flags and poke a self-pipe; `event_wait`
//! polls that pipe together with the watched descriptors.

use std::ffi::CString;
use std::io;
use std::mem::ManuallyDrop;
use std::os::fd::{FromRawFd, IntoRawFd, OwnedFd};
use std::os::unix::fs::DirBuilderExt;
use std::os::unix::net::{UnixListener, UnixStream};
use std::os::unix::process::CommandExt;
use std::path::PathBuf;
use std::sync::{Mutex, Once};
use std::sync::atomic::{AtomicBool, AtomicI32, Ordering};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use siyana_abi::{clock as clk, handle, kill as k, mkdir as mk, open as op, remove as rm};

use crate::event::Event;
use crate::fs::{DirEntry, FileType};
use crate::process::{ExitStatus, Spawn};
use crate::{Error, Handle, Result};

// ---------------------------------------------------------------- errors

fn from_errno(e: i32) -> Error {
    match e {
        libc::ENOENT | libc::ECONNREFUSED => Error::NotFound,
        libc::EACCES | libc::EPERM => Error::PermissionDenied,
        libc::EEXIST | libc::EADDRINUSE => Error::AlreadyExists,
        libc::EINVAL => Error::InvalidInput,
        libc::EAGAIN => Error::WouldBlock,
        libc::EINTR => Error::Interrupted,
        libc::ENOTDIR => Error::NotADirectory,
        libc::EISDIR => Error::IsADirectory,
        libc::ENOMEM => Error::OutOfMemory,
        libc::EBADF => Error::BadHandle,
        libc::ECHILD => Error::NoChild,
        libc::ENOSYS | libc::EOPNOTSUPP => Error::Unsupported,
        libc::EPIPE | libc::ECONNRESET => Error::BrokenPipe,
        libc::ETIMEDOUT => Error::TimedOut,
        libc::ENOTEMPTY => Error::NotEmpty,
        _ => Error::Other(siyana_abi::error::OTHER),
    }
}

fn last_error() -> Error {
    from_errno(io::Error::last_os_error().raw_os_error().unwrap_or(0))
}

fn from_io(e: io::Error) -> Error {
    if let Some(code) = e.raw_os_error() {
        return from_errno(code);
    }
    match e.kind() {
        io::ErrorKind::NotFound => Error::NotFound,
        io::ErrorKind::PermissionDenied => Error::PermissionDenied,
        io::ErrorKind::AlreadyExists => Error::AlreadyExists,
        io::ErrorKind::InvalidInput => Error::InvalidInput,
        io::ErrorKind::Unsupported => Error::Unsupported,
        _ => Error::Other(siyana_abi::error::OTHER),
    }
}

fn cstr(s: &str) -> Result<CString> {
    CString::new(s).map_err(|_| Error::InvalidInput)
}

// ------------------------------------------------------------ basic I/O

pub fn exit(code: i32) -> ! {
    std::process::exit(code)
}

pub fn write(h: Handle, buf: &[u8]) -> Result<usize> {
    let n = unsafe { libc::write(h as i32, buf.as_ptr().cast(), buf.len()) };
    if n < 0 { Err(last_error()) } else { Ok(n as usize) }
}

pub fn read(h: Handle, buf: &mut [u8]) -> Result<usize> {
    let n = unsafe { libc::read(h as i32, buf.as_mut_ptr().cast(), buf.len()) };
    if n < 0 { Err(last_error()) } else { Ok(n as usize) }
}

pub fn close(h: Handle) -> Result<()> {
    if unsafe { libc::close(h as i32) } == 0 { Ok(()) } else { Err(last_error()) }
}

// -------------------------------------------------------- files and dirs

pub fn open(path: &str, flags: usize) -> Result<Handle> {
    if flags & op::DIRECTORY != 0 {
        let p = cstr(path)?;
        let fd = unsafe { libc::open(p.as_ptr(), libc::O_RDONLY | libc::O_DIRECTORY | libc::O_CLOEXEC) };
        return if fd < 0 { Err(last_error()) } else { Ok(fd as Handle) };
    }
    let file = std::fs::OpenOptions::new()
        .read(flags & op::READ != 0)
        .write(flags & op::WRITE != 0)
        .create(flags & op::CREATE != 0)
        .truncate(flags & op::TRUNCATE != 0)
        .append(flags & op::APPEND != 0)
        .open(path)
        .map_err(from_io)?;
    Ok(file.into_raw_fd() as Handle)
}

pub fn mkdir(path: &str, flags: usize) -> Result<()> {
    std::fs::DirBuilder::new()
        .recursive(flags & mk::PARENTS != 0)
        .mode(0o777)
        .create(path)
        .map_err(from_io)
}

pub fn remove(path: &str, flags: usize) -> Result<()> {
    let r = if flags & rm::DIRECTORY == 0 {
        std::fs::remove_file(path)
    } else if flags & rm::RECURSIVE != 0 {
        std::fs::remove_dir_all(path)
    } else {
        std::fs::remove_dir(path)
    };
    r.map_err(from_io)
}

pub fn read_dir(path: &str) -> Result<Vec<DirEntry>> {
    let mut out = Vec::new();
    for entry in std::fs::read_dir(path).map_err(from_io)? {
        let entry = entry.map_err(from_io)?;
        let ft = entry.file_type().map_err(from_io)?;
        out.push(DirEntry {
            name: entry.file_name().to_string_lossy().into_owned(),
            file_type: if ft.is_dir() {
                FileType::Directory
            } else if ft.is_file() {
                FileType::File
            } else {
                FileType::Other
            },
        });
    }
    Ok(out)
}

// ------------------------------------------------------------------ time

pub fn clock(id: usize) -> u64 {
    if id == clk::REALTIME {
        let d = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default();
        return d.as_nanos() as u64;
    }
    let mut ts = libc::timespec { tv_sec: 0, tv_nsec: 0 };
    unsafe { libc::clock_gettime(libc::CLOCK_MONOTONIC, &mut ts) };
    ts.tv_sec as u64 * 1_000_000_000 + ts.tv_nsec as u64
}

pub fn sleep(ns: u64) {
    std::thread::sleep(Duration::from_nanos(ns));
}

// ------------------------------------------------------------- processes

pub fn getpid() -> u32 {
    std::process::id()
}

fn child_stdio(h: u32) -> Result<std::process::Stdio> {
    Ok(match h {
        handle::INHERIT => std::process::Stdio::inherit(),
        handle::NULL => std::process::Stdio::null(),
        fd => {
            // The child gets its own copy; the parent keeps its handle.
            let dup = unsafe { libc::fcntl(fd as i32, libc::F_DUPFD_CLOEXEC, 0) };
            if dup < 0 {
                return Err(last_error());
            }
            std::process::Stdio::from(unsafe { OwnedFd::from_raw_fd(dup) })
        }
    })
}

pub(crate) fn spawn(s: &Spawn) -> Result<u32> {
    ensure_child_events();
    let mut cmd = std::process::Command::new(s.path);
    if let Some((argv0, rest)) = s.args.split_first() {
        cmd.arg0(argv0).args(rest);
    }
    cmd.env_clear().envs(s.env.iter().map(|(k, v)| (k, v)));
    if let Some(dir) = s.cwd {
        cmd.current_dir(dir);
    }
    cmd.stdin(child_stdio(s.stdio[0])?)
        .stdout(child_stdio(s.stdio[1])?)
        .stderr(child_stdio(s.stdio[2])?);
    if s.new_group {
        cmd.process_group(0);
    }
    // Dropping the Child neither waits nor kills: callers reap by pid.
    let child = cmd.spawn().map_err(from_io)?;
    Ok(child.id())
}

pub fn kill(pid: u32, how: usize) -> Result<()> {
    let sig = match how & 0xff {
        k::TERMINATE => libc::SIGTERM,
        k::KILL => libc::SIGKILL,
        _ => return Err(Error::InvalidInput),
    };
    let target = if how & k::GROUP != 0 { -(pid as i32) } else { pid as i32 };
    if unsafe { libc::kill(target, sig) } == 0 { Ok(()) } else { Err(last_error()) }
}

fn exit_status(raw: i32) -> ExitStatus {
    if libc::WIFEXITED(raw) {
        ExitStatus::Code(libc::WEXITSTATUS(raw))
    } else {
        match libc::WTERMSIG(raw) {
            // Deliberate stops; everything else (SIGSEGV, SIGFPE, ...) is a fault.
            libc::SIGKILL | libc::SIGTERM | libc::SIGINT | libc::SIGHUP => ExitStatus::Killed,
            _ => ExitStatus::Crashed,
        }
    }
}

pub fn wait(pid: u32, nohang: bool) -> Result<Option<(u32, ExitStatus)>> {
    let target = if pid == siyana_abi::wait::ANY as u32 { -1 } else { pid as i32 };
    let flags = if nohang { libc::WNOHANG } else { 0 };
    loop {
        let mut raw = 0;
        let r = unsafe { libc::waitpid(target, &mut raw, flags) };
        if r > 0 {
            return Ok(Some((r as u32, exit_status(raw))));
        }
        if r == 0 {
            return Ok(None);
        }
        let e = last_error();
        if e != Error::Interrupted {
            return Err(e);
        }
    }
}

pub fn set_reaper() -> Result<()> {
    if unsafe { libc::prctl(libc::PR_SET_CHILD_SUBREAPER, 1, 0, 0, 0) } == 0 {
        Ok(())
    } else {
        Err(last_error())
    }
}

// ---------------------------------------------------------------- events

static SHUTDOWN: AtomicBool = AtomicBool::new(false);
static CHILD: AtomicBool = AtomicBool::new(false);
static WAKE_R: AtomicI32 = AtomicI32::new(-1);
static WAKE_W: AtomicI32 = AtomicI32::new(-1);
static PIPE_ONCE: Once = Once::new();
static CHILD_ONCE: Once = Once::new();
static SHUTDOWN_ONCE: Once = Once::new();

// Signal handlers: only async-signal-safe work (atomics + write(2)).
fn poke() {
    let w = WAKE_W.load(Ordering::SeqCst);
    if w >= 0 {
        unsafe {
            let errno = *libc::__errno_location();
            libc::write(w, [1u8].as_ptr().cast(), 1);
            *libc::__errno_location() = errno;
        }
    }
}

extern "C" fn on_shutdown(_: libc::c_int) {
    SHUTDOWN.store(true, Ordering::SeqCst);
    poke();
}

extern "C" fn on_child(_: libc::c_int) {
    CHILD.store(true, Ordering::SeqCst);
    poke();
}

fn wake_pipe() {
    PIPE_ONCE.call_once(|| {
        let mut fds = [0; 2];
        if unsafe { libc::pipe2(fds.as_mut_ptr(), libc::O_NONBLOCK | libc::O_CLOEXEC) } == 0 {
            WAKE_R.store(fds[0], Ordering::SeqCst);
            WAKE_W.store(fds[1], Ordering::SeqCst);
        }
    });
}

fn install(sig: libc::c_int, handler: extern "C" fn(libc::c_int), flags: libc::c_int) {
    unsafe {
        let mut action: libc::sigaction = std::mem::zeroed();
        action.sa_sigaction = handler as usize;
        action.sa_flags = flags;
        libc::sigemptyset(&mut action.sa_mask);
        libc::sigaction(sig, &action, std::ptr::null_mut());
    }
}

/// Child exits become `ChildExited` events. Installed on first spawn or
/// event wait; reports one event up front in case a child already exited.
fn ensure_child_events() {
    CHILD_ONCE.call_once(|| {
        wake_pipe();
        install(libc::SIGCHLD, on_child, libc::SA_RESTART | libc::SA_NOCLDSTOP);
        CHILD.store(true, Ordering::SeqCst);
    });
}

/// SIGTERM / SIGINT become `Shutdown` events instead of killing us.
fn ensure_shutdown_events() {
    SHUTDOWN_ONCE.call_once(|| {
        wake_pipe();
        install(libc::SIGTERM, on_shutdown, libc::SA_RESTART);
        install(libc::SIGINT, on_shutdown, libc::SA_RESTART);
    });
}

pub fn event_wait(timeout_ns: u64, watch: &[Handle]) -> Result<Vec<Event>> {
    ensure_child_events();
    ensure_shutdown_events();
    let deadline = (timeout_ns != u64::MAX).then(|| Instant::now() + Duration::from_nanos(timeout_ns));
    let wake_r = WAKE_R.load(Ordering::SeqCst);

    loop {
        let pending = SHUTDOWN.load(Ordering::SeqCst) || CHILD.load(Ordering::SeqCst);
        let timeout_ms: i32 = match deadline {
            _ if pending => 0,
            None => -1,
            Some(d) => {
                let left = d.saturating_duration_since(Instant::now());
                left.as_nanos().div_ceil(1_000_000).min(i32::MAX as u128) as i32
            }
        };

        let mut fds: Vec<libc::pollfd> = std::iter::once(wake_r)
            .chain(watch.iter().map(|&h| h as i32))
            .map(|fd| libc::pollfd { fd, events: libc::POLLIN, revents: 0 })
            .collect();
        let r = unsafe { libc::poll(fds.as_mut_ptr(), fds.len() as libc::nfds_t, timeout_ms) };
        if r < 0 && last_error() != Error::Interrupted {
            return Err(last_error());
        }

        if fds[0].revents != 0 {
            let mut sink = [0u8; 64];
            while unsafe { libc::read(wake_r, sink.as_mut_ptr().cast(), sink.len()) } > 0 {}
        }

        let mut events = Vec::new();
        if SHUTDOWN.swap(false, Ordering::SeqCst) {
            events.push(Event::Shutdown);
        }
        if CHILD.swap(false, Ordering::SeqCst) {
            events.push(Event::ChildExited);
        }
        let ready = libc::POLLIN | libc::POLLHUP | libc::POLLERR;
        for (pfd, &h) in fds[1..].iter().zip(watch) {
            if pfd.revents & ready != 0 {
                events.push(Event::Readable(h));
            }
        }

        let timed_out = deadline.is_some_and(|d| Instant::now() >= d);
        if !events.is_empty() || timed_out {
            return Ok(events);
        }
    }
}

// ------------------------------------------------------------------- IPC

/// IPC names map to Unix sockets under one private directory:
/// `$SIYANA_IPC_DIR`, else `$XDG_RUNTIME_DIR/siyana-ipc`, else
/// `/tmp/siyana-ipc-<uid>`. Sockets live in `sock/<name>`; each live
/// listener holds an `flock` on `lock/<name>`, which is how a stale socket
/// left by a dead process is told apart from a live one (the kernel drops
/// the lock when the process dies).
fn ipc_paths(name: &str) -> Result<(PathBuf, PathBuf)> {
    let dir = if let Some(d) = std::env::var_os("SIYANA_IPC_DIR") {
        PathBuf::from(d)
    } else if let Some(d) = std::env::var_os("XDG_RUNTIME_DIR") {
        PathBuf::from(d).join("siyana-ipc")
    } else {
        PathBuf::from(format!("/tmp/siyana-ipc-{}", unsafe { libc::getuid() }))
    };
    for sub in ["sock", "lock"] {
        std::fs::DirBuilder::new().recursive(true).mode(0o700).create(dir.join(sub)).map_err(from_io)?;
    }
    Ok((dir.join("sock").join(name), dir.join("lock").join(name)))
}

/// Listener handle -> its lock file descriptor.
static LISTENER_LOCKS: Mutex<Vec<(Handle, i32)>> = Mutex::new(Vec::new());

pub fn ipc_listen(name: &str) -> Result<Handle> {
    let (sock, lock) = ipc_paths(name)?;
    let lock_fd = unsafe { libc::open(cstr(lock.to_str().ok_or(Error::InvalidInput)?)?.as_ptr(), libc::O_RDWR | libc::O_CREAT | libc::O_CLOEXEC, 0o600) };
    if lock_fd < 0 {
        return Err(last_error());
    }
    if unsafe { libc::flock(lock_fd, libc::LOCK_EX | libc::LOCK_NB) } != 0 {
        unsafe { libc::close(lock_fd) };
        return Err(Error::AlreadyExists);
    }
    // We hold the lock, so any socket file there is stale.
    let _ = std::fs::remove_file(&sock);
    match UnixListener::bind(&sock) {
        Ok(l) => {
            let h = l.into_raw_fd() as Handle;
            LISTENER_LOCKS.lock().unwrap_or_else(|e| e.into_inner()).push((h, lock_fd));
            Ok(h)
        }
        Err(e) => {
            unsafe { libc::close(lock_fd) };
            Err(from_io(e))
        }
    }
}

pub fn ipc_accept(h: Handle) -> Result<Handle> {
    let listener = ManuallyDrop::new(unsafe { UnixListener::from_raw_fd(h as i32) });
    let (stream, _) = listener.accept().map_err(from_io)?;
    Ok(stream.into_raw_fd() as Handle)
}

pub fn ipc_connect(name: &str) -> Result<Handle> {
    let (sock, _) = ipc_paths(name)?;
    Ok(UnixStream::connect(sock).map_err(from_io)?.into_raw_fd() as Handle)
}

pub fn ipc_shutdown_write(h: Handle) -> Result<()> {
    if unsafe { libc::shutdown(h as i32, libc::SHUT_WR) } == 0 { Ok(()) } else { Err(last_error()) }
}

/// Remove the socket file (while still holding the lock, so no new
/// listener can be racing us), then close the listener and drop the lock.
pub fn ipc_unlisten(h: Handle, name: &str) {
    if let Ok((sock, _)) = ipc_paths(name) {
        let _ = std::fs::remove_file(sock);
    }
    let _ = close(h);
    let mut locks = LISTENER_LOCKS.lock().unwrap_or_else(|e| e.into_inner());
    if let Some(i) = locks.iter().position(|&(lh, _)| lh == h) {
        let (_, lock_fd) = locks.swap_remove(i);
        unsafe { libc::close(lock_fd) };
    }
}

// ---------------------------------------------------------- args and env

pub fn args() -> Vec<String> {
    std::env::args_os().map(|a| a.to_string_lossy().into_owned()).collect()
}

pub fn env() -> Vec<(String, String)> {
    std::env::vars_os()
        .map(|(k, v)| (k.to_string_lossy().into_owned(), v.to_string_lossy().into_owned()))
        .collect()
}
