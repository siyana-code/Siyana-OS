//! One test: the event queue is process-wide, and reap() collects any child.

use libsiyana::event::{self, Event};
use libsiyana::ipc::{Listener, Stream};
use libsiyana::process::{self, Command, ExitStatus, Signal};
use libsiyana::time::{Duration, Instant};

fn wait_for(want: impl Fn(&Event) -> bool, watch: &[u32]) -> Vec<Event> {
    let deadline = Instant::now() + Duration::from_secs(5);
    let mut seen = Vec::new();
    while Instant::now() < deadline {
        let evs = event::wait(Some(Duration::from_millis(500)), watch).unwrap();
        let hit = evs.iter().any(&want);
        seen.extend(evs);
        if hit {
            return seen;
        }
    }
    panic!("event never arrived; saw {seen:?}");
}

#[test]
fn event_queue() {
    // A pure timeout returns an empty list after roughly the timeout.
    let _ = event::wait(Some(Duration::ZERO), &[]); // drain the initial ChildExited
    let t = Instant::now();
    assert_eq!(event::wait(Some(Duration::from_millis(100)), &[]).unwrap(), []);
    assert!(t.elapsed() >= Duration::from_millis(100));

    // Child exit -> ChildExited, then reap() collects it.
    let pid = Command::new("/bin/sh").args(["-c", "exit 7"]).spawn().unwrap();
    wait_for(|e| *e == Event::ChildExited, &[]);
    let reaped = {
        let mut all = process::reap();
        let deadline = Instant::now() + Duration::from_secs(2);
        while !all.iter().any(|(p, _)| *p == pid) && Instant::now() < deadline {
            all.extend(process::reap());
        }
        all
    };
    assert!(reaped.contains(&(pid, ExitStatus::Code(7))), "{reaped:?}");
    assert_eq!(process::try_wait_any().unwrap(), None);

    // Terminate after opting in -> Shutdown event, and we keep running.
    process::kill(process::id(), Signal::Terminate).unwrap();
    wait_for(|e| *e == Event::Shutdown, &[]);

    // A connection on a watched listener -> Readable(listener).
    let dir = std::env::temp_dir().join(format!("libsiyana-ev-{}", std::process::id()));
    unsafe { std::env::set_var("SIYANA_IPC_DIR", &dir) };
    let l = Listener::bind("events-test").unwrap();
    let _client = Stream::connect("events-test").unwrap();
    wait_for(|e| *e == Event::Readable(l.handle()), &[l.handle()]);
    drop(l);
    let _ = std::fs::remove_dir_all(dir);
}
