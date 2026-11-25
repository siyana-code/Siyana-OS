//! Processes: spawn, stop, wait, exit.
//!
//! Siyana has no POSIX signal handlers. `Signal::Terminate` asks a process
//! to stop by sending it an `Event::Shutdown` (see [`crate::event`]);
//! `Signal::Kill` ends it immediately. A process that has never called
//! `event::wait` is simply killed by `Terminate`.

use alloc::string::{String, ToString};
use alloc::vec::Vec;
use core::fmt;

use siyana_abi::{handle, kill as k};

use crate::{backend, env, Handle, Result};

pub type Pid = u32;

pub fn exit(code: i32) -> ! {
    backend::exit(code)
}

/// This process's id.
pub fn id() -> Pid {
    backend::getpid()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Signal {
    /// Ask politely: the target gets `Event::Shutdown`.
    Terminate,
    /// Stop now. Cannot be caught or ignored.
    Kill,
}

impl Signal {
    fn abi(self) -> usize {
        match self {
            Signal::Terminate => k::TERMINATE,
            Signal::Kill => k::KILL,
        }
    }
}

/// Send `sig` to one process.
pub fn kill(pid: Pid, sig: Signal) -> Result<()> {
    backend::kill(pid, sig.abi())
}

/// Send `sig` to every process in the group led by `pid` (see
/// [`Command::new_process_group`]).
pub fn kill_group(pid: Pid, sig: Signal) -> Result<()> {
    backend::kill(pid, sig.abi() | k::GROUP)
}

/// How a process ended.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExitStatus {
    /// It called `exit(code)` (or returned `code` from main).
    Code(i32),
    /// Stopped by `Signal::Kill`, or by `Terminate` without handling it.
    Killed,
    /// The OS killed it after a fault (bad memory access, ...).
    Crashed,
}

impl ExitStatus {
    pub fn success(self) -> bool {
        self == ExitStatus::Code(0)
    }
}

impl fmt::Display for ExitStatus {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ExitStatus::Code(c) => write!(f, "exited with code {c}"),
            ExitStatus::Killed => f.write_str("killed"),
            ExitStatus::Crashed => f.write_str("crashed"),
        }
    }
}

/// Wait for a specific child to finish.
pub fn wait(pid: Pid) -> Result<ExitStatus> {
    loop {
        if let Some((_, status)) = backend::wait(pid, false)? {
            return Ok(status);
        }
    }
}

/// Has this child finished? Never blocks.
pub fn try_wait(pid: Pid) -> Result<Option<ExitStatus>> {
    Ok(backend::wait(pid, true)?.map(|(_, s)| s))
}

/// Collect any one finished child, without blocking.
/// `Ok(None)` when nothing has finished (or there are no children).
pub fn try_wait_any() -> Result<Option<(Pid, ExitStatus)>> {
    match backend::wait(siyana_abi::wait::ANY as Pid, true) {
        Err(crate::Error::NoChild) => Ok(None),
        other => other,
    }
}

/// Collect every finished child, without blocking.
pub fn reap() -> Vec<(Pid, ExitStatus)> {
    let mut done = Vec::new();
    while let Ok(Some(c)) = try_wait_any() {
        done.push(c);
    }
    done
}

/// Become the reaper for orphaned descendants: when a child's own children
/// are orphaned, they are re-parented to us instead of being lost.
/// Init and service managers want this.
pub fn become_reaper() -> Result<()> {
    backend::set_reaper()
}

/// Where a child's stdin/stdout/stderr goes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Stdio {
    /// Share the parent's stream.
    Inherit,
    /// Nothing: reads see end of input, writes are discarded.
    Null,
    /// A handle the parent owns (file, IPC stream...).
    Handle(Handle),
}

impl Stdio {
    fn abi(self) -> u32 {
        match self {
            Stdio::Inherit => handle::INHERIT,
            Stdio::Null => handle::NULL,
            Stdio::Handle(h) => h,
        }
    }
}

/// Everything the backends need to start a process.
pub(crate) struct Spawn<'a> {
    pub path: &'a str,
    /// Full argv, including argv[0].
    pub args: &'a [String],
    /// The child's complete environment.
    pub env: &'a [(String, String)],
    pub cwd: Option<&'a str>,
    /// `handle::*` values or real handles, for stdin/stdout/stderr.
    pub stdio: [u32; 3],
    pub new_group: bool,
}

/// Builder for starting a child process.
///
/// ```ignore
/// let pid = Command::new("/bin/logd").arg("--verbose").env("LEVEL", "2").spawn()?;
/// ```
#[derive(Debug, Clone)]
pub struct Command {
    program: String,
    args: Vec<String>,
    env: Vec<(String, String)>,
    clear_env: bool,
    cwd: Option<String>,
    stdio: [Stdio; 3],
    new_group: bool,
}

impl Command {
    pub fn new(program: &str) -> Command {
        Command {
            program: program.to_string(),
            args: alloc::vec![program.to_string()],
            env: Vec::new(),
            clear_env: false,
            cwd: None,
            stdio: [Stdio::Inherit; 3],
            new_group: false,
        }
    }

    pub fn arg(&mut self, a: &str) -> &mut Self {
        self.args.push(a.to_string());
        self
    }

    pub fn args<I, S>(&mut self, args: I) -> &mut Self
    where
        I: IntoIterator<Item = S>,
        S: AsRef<str>,
    {
        self.args.extend(args.into_iter().map(|a| a.as_ref().to_string()));
        self
    }

    /// Set one environment variable for the child (on top of ours).
    pub fn env(&mut self, key: &str, val: &str) -> &mut Self {
        self.env.retain(|(k, _)| k != key);
        self.env.push((key.to_string(), val.to_string()));
        self
    }

    pub fn envs<I, K, V>(&mut self, vars: I) -> &mut Self
    where
        I: IntoIterator<Item = (K, V)>,
        K: AsRef<str>,
        V: AsRef<str>,
    {
        for (k, v) in vars {
            self.env(k.as_ref(), v.as_ref());
        }
        self
    }

    /// Start the child with no inherited environment, only what `env` adds.
    pub fn env_clear(&mut self) -> &mut Self {
        self.clear_env = true;
        self.env.clear();
        self
    }

    pub fn current_dir(&mut self, dir: &str) -> &mut Self {
        self.cwd = Some(dir.to_string());
        self
    }

    pub fn stdin(&mut self, s: Stdio) -> &mut Self {
        self.stdio[0] = s;
        self
    }

    pub fn stdout(&mut self, s: Stdio) -> &mut Self {
        self.stdio[1] = s;
        self
    }

    pub fn stderr(&mut self, s: Stdio) -> &mut Self {
        self.stdio[2] = s;
        self
    }

    /// Run the child in a new process group led by itself, so
    /// [`kill_group`] reaches anything it starts too.
    pub fn new_process_group(&mut self, on: bool) -> &mut Self {
        self.new_group = on;
        self
    }

    /// Start the child and return its pid. It keeps running on its own;
    /// collect it later with [`wait`], [`try_wait`] or [`reap`].
    pub fn spawn(&self) -> Result<Pid> {
        let mut env = if self.clear_env { Vec::new() } else { env::vars() };
        for (k, v) in &self.env {
            env.retain(|(ek, _)| ek != k);
            env.push((k.clone(), v.clone()));
        }
        backend::spawn(&Spawn {
            path: &self.program,
            args: &self.args,
            env: &env,
            cwd: self.cwd.as_deref(),
            stdio: self.stdio.map(Stdio::abi),
            new_group: self.new_group,
        })
    }

    /// Start the child and wait for it to finish.
    pub fn status(&self) -> Result<ExitStatus> {
        wait(self.spawn()?)
    }
}
