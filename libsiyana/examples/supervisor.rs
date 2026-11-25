//! A tiny service supervisor: the event-loop pattern Siyana uses instead of
//! POSIX signal handlers. Starts two "services", restarts one when it dies,
//! answers `status` / `stop` over IPC, and shuts down cleanly on Ctrl-C.
//!
//!   cargo run --example supervisor            # terminal 1
//!   cargo run --example supervisor -- status  # terminal 2
//!   cargo run --example supervisor -- stop

#![cfg_attr(not(target_os = "linux"), no_std, no_main)]

use libsiyana::event::{self, Event};
use libsiyana::ipc::{Listener, Stream};
use libsiyana::prelude::*;
use libsiyana::process::{self, Command, ExitStatus, Pid, Signal};
use libsiyana::{env, time};

libsiyana::entry!(app_main);

const CONTROL: &str = "supervisor-example";

struct Service {
    name: &'static str,
    argv: &'static [&'static str],
    pid: Option<Pid>,
    restarts: u32,
}

fn app_main() -> i32 {
    match env::args().get(1).map(String::as_str) {
        None => run(),
        Some(cmd @ ("status" | "stop")) => client(cmd),
        Some(other) => {
            eprintln!("unknown command {other:?}; use status or stop");
            2
        }
    }
}

fn client(cmd: &str) -> i32 {
    let mut s = match Stream::connect(CONTROL) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("cannot reach supervisor: {e}");
            return 1;
        }
    };
    let mut reply = Vec::new();
    let ok = s.write_all(cmd.as_bytes()).and_then(|_| s.shutdown_write()).and_then(|_| s.read_to_end(&mut reply));
    if let Err(e) = ok {
        eprintln!("control request failed: {e}");
        return 1;
    }
    print!("{}", String::from_utf8_lossy(&reply));
    0
}

fn start(svc: &mut Service) {
    match Command::new(svc.argv[0]).args(&svc.argv[1..]).new_process_group(true).spawn() {
        Ok(pid) => {
            println!("[supervisor] started {} (pid {pid})", svc.name);
            svc.pid = Some(pid);
        }
        Err(e) => eprintln!("[supervisor] cannot start {}: {e}", svc.name),
    }
}

fn run() -> i32 {
    let listener = match Listener::bind(CONTROL) {
        Ok(l) => l,
        Err(e) => {
            eprintln!("cannot bind control endpoint: {e}");
            return 1;
        }
    };
    let _ = process::become_reaper();

    let mut services = [
        Service { name: "ticker", argv: &["/bin/sh", "-c", "while true; do sleep 2; done"], pid: None, restarts: 0 },
        Service { name: "flaky", argv: &["/bin/sh", "-c", "sleep 3; exit 1"], pid: None, restarts: 0 },
    ];
    for svc in services.iter_mut() {
        start(svc);
    }

    let mut stopping = false;
    while !stopping {
        let events = match event::wait(Some(Duration::from_secs(5)), &[listener.handle()]) {
            Ok(evs) => evs,
            Err(e) => {
                eprintln!("[supervisor] event wait failed: {e}");
                break;
            }
        };
        for ev in events {
            match ev {
                Event::Shutdown => stopping = true,
                Event::ChildExited => {
                    for (pid, status) in process::reap() {
                        if let Some(svc) = services.iter_mut().find(|s| s.pid == Some(pid)) {
                            println!("[supervisor] {} {status}", svc.name);
                            svc.pid = None;
                            if !matches!(status, ExitStatus::Code(0)) {
                                svc.restarts += 1;
                                start(svc);
                            }
                        }
                    }
                }
                Event::Readable(_) => {
                    if let Ok(mut client) = listener.accept() {
                        let mut req = Vec::new();
                        let _ = client.read_to_end(&mut req);
                        let reply = match req.as_slice() {
                            b"status" => services
                                .iter()
                                .map(|s| format!("{:<8} pid={:<8} restarts={}\n", s.name, s.pid.map_or("-".to_string(), |p| p.to_string()), s.restarts))
                                .collect::<String>(),
                            b"stop" => {
                                stopping = true;
                                "stopping\n".to_string()
                            }
                            _ => "unknown request\n".to_string(),
                        };
                        let _ = client.write_all(reply.as_bytes());
                    }
                }
            }
        }
    }

    println!("[supervisor] shutting down");
    for svc in services.iter() {
        if let Some(pid) = svc.pid {
            let _ = process::kill_group(pid, Signal::Terminate);
        }
    }
    let deadline = time::Instant::now() + Duration::from_secs(3);
    while services.iter().any(|s| s.pid.is_some()) && time::Instant::now() < deadline {
        for (pid, status) in process::reap() {
            if let Some(svc) = services.iter_mut().find(|s| s.pid == Some(pid)) {
                println!("[supervisor] {} {status}", svc.name);
                svc.pid = None;
            }
        }
        time::sleep(Duration::from_millis(50));
    }
    for svc in services.iter().filter(|s| s.pid.is_some()) {
        let _ = process::kill_group(svc.pid.unwrap(), Signal::Kill);
    }
    0
}
