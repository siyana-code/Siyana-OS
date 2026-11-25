//! Siyana backend: native system calls, exactly as specified in
//! `siyana-abi` / `docs/abi.md`. Until the kernel implements a call it
//! returns `Error::Unsupported` (or the process faults, before Phase 4).

use alloc::string::String;
use alloc::vec;
use alloc::vec::Vec;
use core::arch::asm;

use siyana_abi::{self as abi, nr, DirRecord, ExitInfo, SpawnArgs, Str};

use crate::event::Event;
use crate::fs::{DirEntry, FileType};
use crate::process::{ExitStatus, Spawn};
use crate::{Error, Handle, Result};

// ------------------------------------------------------- raw syscalls

#[inline(always)]
unsafe fn syscall(n: usize, a1: usize, a2: usize, a3: usize, a4: usize, a5: usize) -> isize {
    let ret: isize;
    unsafe {
        asm!(
            "syscall",
            inlateout("rax") n as isize => ret,
            in("rdi") a1,
            in("rsi") a2,
            in("rdx") a3,
            in("r10") a4,
            in("r8") a5,
            lateout("rcx") _,
            lateout("r11") _,
            options(nostack),
        );
    }
    ret
}

fn check(r: isize) -> Result<usize> {
    if r < 0 { Err(Error::from_code(-r)) } else { Ok(r as usize) }
}

fn call(n: usize, a1: usize, a2: usize, a3: usize, a4: usize, a5: usize) -> Result<usize> {
    check(unsafe { syscall(n, a1, a2, a3, a4, a5) })
}

fn s(x: &str) -> Str {
    Str { ptr: x.as_ptr(), len: x.len() }
}

// ------------------------------------------------------------ basic I/O

pub fn exit(code: i32) -> ! {
    unsafe { syscall(nr::EXIT, code as usize, 0, 0, 0, 0) };
    // EXIT never returns; if a broken kernel lets it, don't run on.
    loop {
        core::hint::spin_loop();
    }
}

pub fn write(h: Handle, buf: &[u8]) -> Result<usize> {
    call(nr::WRITE, h as usize, buf.as_ptr() as usize, buf.len(), 0, 0)
}

pub fn read(h: Handle, buf: &mut [u8]) -> Result<usize> {
    call(nr::READ, h as usize, buf.as_mut_ptr() as usize, buf.len(), 0, 0)
}

pub fn close(h: Handle) -> Result<()> {
    call(nr::CLOSE, h as usize, 0, 0, 0, 0).map(drop)
}

// -------------------------------------------------------- files and dirs

pub fn open(path: &str, flags: usize) -> Result<Handle> {
    call(nr::OPEN, path.as_ptr() as usize, path.len(), flags, 0, 0).map(|h| h as Handle)
}

pub fn mkdir(path: &str, flags: usize) -> Result<()> {
    call(nr::MKDIR, path.as_ptr() as usize, path.len(), flags, 0, 0).map(drop)
}

pub fn remove(path: &str, flags: usize) -> Result<()> {
    call(nr::REMOVE, path.as_ptr() as usize, path.len(), flags, 0, 0).map(drop)
}

pub fn read_dir(path: &str) -> Result<Vec<DirEntry>> {
    let h = open(path, abi::open::DIRECTORY)?;
    let mut out = Vec::new();
    let mut buf = vec![0u8; 4096];
    let result = loop {
        let n = match call(nr::READ_DIR, h as usize, buf.as_mut_ptr() as usize, buf.len(), 0, 0) {
            Ok(0) => break Ok(()),
            Ok(n) => n,
            Err(e) => break Err(e),
        };
        let hdr = core::mem::size_of::<DirRecord>();
        let mut at = 0;
        while at + hdr <= n {
            let rec: DirRecord = unsafe { core::ptr::read_unaligned(buf[at..].as_ptr().cast()) };
            let name_end = at + hdr + rec.name_len as usize;
            if name_end > n {
                break;
            }
            let name = core::str::from_utf8(&buf[at + hdr..name_end]).map_err(|_| Error::InvalidUtf8)?;
            out.push(DirEntry {
                name: String::from(name),
                file_type: match rec.kind {
                    abi::dir_kind::FILE => FileType::File,
                    abi::dir_kind::DIRECTORY => FileType::Directory,
                    _ => FileType::Other,
                },
            });
            at = name_end;
        }
    };
    let _ = close(h);
    result.map(|()| out)
}

// ------------------------------------------------------------------ time

pub fn clock(id: usize) -> u64 {
    call(nr::CLOCK, id, 0, 0, 0, 0).unwrap_or(0) as u64
}

pub fn sleep(ns: u64) {
    let _ = call(nr::SLEEP, ns as usize, 0, 0, 0, 0);
}

// ------------------------------------------------------------- processes

pub fn getpid() -> u32 {
    call(nr::GETPID, 0, 0, 0, 0, 0).unwrap_or(0) as u32
}

pub(crate) fn spawn(sp: &Spawn) -> Result<u32> {
    let argv: Vec<Str> = sp.args.iter().map(|a| s(a)).collect();
    let env_strings: Vec<String> = sp.env.iter().map(|(k, v)| alloc::format!("{k}={v}")).collect();
    let envp: Vec<Str> = env_strings.iter().map(|e| s(e)).collect();
    let args = SpawnArgs {
        path: s(sp.path),
        argv: argv.as_ptr(),
        argc: argv.len(),
        envp: envp.as_ptr(),
        envc: envp.len(),
        cwd: s(sp.cwd.unwrap_or("")),
        stdio: sp.stdio,
        flags: if sp.new_group { abi::spawn::NEW_GROUP } else { 0 },
    };
    call(nr::SPAWN, &args as *const SpawnArgs as usize, 0, 0, 0, 0).map(|p| p as u32)
}

pub fn kill(pid: u32, how: usize) -> Result<()> {
    call(nr::KILL, pid as usize, how, 0, 0, 0).map(drop)
}

pub fn wait(pid: u32, nohang: bool) -> Result<Option<(u32, ExitStatus)>> {
    let mut info = ExitInfo::default();
    let flags = if nohang { abi::wait::NOHANG } else { 0 };
    let got = call(nr::WAIT, pid as usize, flags, &mut info as *mut ExitInfo as usize, 0, 0)?;
    if got == 0 {
        return Ok(None);
    }
    let status = match info.kind {
        abi::exit_kind::CODE => ExitStatus::Code(info.code),
        abi::exit_kind::KILLED => ExitStatus::Killed,
        _ => ExitStatus::Crashed,
    };
    Ok(Some((got as u32, status)))
}

pub fn set_reaper() -> Result<()> {
    call(nr::SET_REAPER, 0, 0, 0, 0, 0).map(drop)
}

// ---------------------------------------------------------------- events

pub fn event_wait(timeout_ns: u64, watch: &[Handle]) -> Result<Vec<Event>> {
    let mut raw = vec![abi::Event::default(); 16 + watch.len()];
    let n = call(
        nr::EVENT_WAIT,
        timeout_ns as usize,
        watch.as_ptr() as usize,
        watch.len(),
        raw.as_mut_ptr() as usize,
        raw.len(),
    )?;
    Ok(raw[..n]
        .iter()
        .filter_map(|e| match e.kind {
            abi::event::SHUTDOWN => Some(Event::Shutdown),
            abi::event::CHILD_EXITED => Some(Event::ChildExited),
            abi::event::READABLE => Some(Event::Readable(e.handle)),
            _ => None,
        })
        .collect())
}

// ------------------------------------------------------------------- IPC

pub fn ipc_listen(name: &str) -> Result<Handle> {
    call(nr::IPC_LISTEN, name.as_ptr() as usize, name.len(), 0, 0, 0).map(|h| h as Handle)
}

pub fn ipc_accept(h: Handle) -> Result<Handle> {
    call(nr::IPC_ACCEPT, h as usize, 0, 0, 0, 0).map(|h| h as Handle)
}

pub fn ipc_connect(name: &str) -> Result<Handle> {
    call(nr::IPC_CONNECT, name.as_ptr() as usize, name.len(), 0, 0, 0).map(|h| h as Handle)
}

pub fn ipc_shutdown_write(h: Handle) -> Result<()> {
    call(nr::IPC_SHUTDOWN_WRITE, h as usize, 0, 0, 0, 0).map(drop)
}

/// Closing the listener handle unregisters the name in the kernel.
pub fn ipc_unlisten(h: Handle, _name: &str) {
    let _ = close(h);
}

// ---------------------------------------------------------- args and env

/// ARGS / ENV: ask for the size, then fetch into a buffer that fits.
fn zero_separated(n: usize) -> Vec<String> {
    let Ok(total) = call(n, 0, 0, 0, 0, 0) else { return Vec::new() };
    let mut buf = vec![0u8; total];
    if call(n, buf.as_mut_ptr() as usize, buf.len(), 0, 0, 0).is_err() {
        return Vec::new();
    }
    // Every entry ends in a 0 byte; strip the last one so an empty
    // argument ("") still counts as an entry.
    match buf.strip_suffix(&[0]) {
        None => Vec::new(),
        Some(body) => body
            .split(|&b| b == 0)
            .map(|part| String::from_utf8_lossy(part).into_owned())
            .collect(),
    }
}

pub fn args() -> Vec<String> {
    zero_separated(nr::ARGS)
}

pub fn env() -> Vec<(String, String)> {
    zero_separated(nr::ENV)
        .into_iter()
        .filter_map(|e| e.split_once('=').map(|(k, v)| (String::from(k), String::from(v))))
        .collect()
}

// ---------------------------------------------------------------- memory

pub fn mem_map(len: usize) -> Result<*mut u8> {
    call(nr::MEM_MAP, len, 0, 0, 0, 0).map(|a| a as *mut u8)
}
