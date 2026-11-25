//! Process tests wait on specific pids only, so they can run in parallel.

use libsiyana::fs::{self, File};
use libsiyana::process::{self, Command, ExitStatus, Signal, Stdio};
use libsiyana::time::{self, Duration, Instant};

fn sh(script: &str) -> Command {
    let mut c = Command::new("/bin/sh");
    c.args(["-c", script]);
    c
}

#[test]
fn exit_codes() {
    assert_eq!(sh("exit 0").status().unwrap(), ExitStatus::Code(0));
    assert_eq!(sh("exit 3").status().unwrap(), ExitStatus::Code(3));
    assert_eq!(sh("ulimit -c 0; kill -SEGV $$").status().unwrap(), ExitStatus::Crashed);
    assert!(Command::new("/no/such/program").spawn().is_err());
}

#[test]
fn terminate_and_kill() {
    let pid = sh("exec sleep 30").spawn().unwrap();
    assert_eq!(process::try_wait(pid).unwrap(), None);
    process::kill(pid, Signal::Terminate).unwrap();
    assert_eq!(process::wait(pid).unwrap(), ExitStatus::Killed);

    // Group kill reaches the grandchild too.
    let pid = sh("sleep 30 & wait").new_process_group(true).spawn().unwrap();
    time::sleep(Duration::from_millis(100));
    process::kill_group(pid, Signal::Kill).unwrap();
    assert_eq!(process::wait(pid).unwrap(), ExitStatus::Killed);
}

#[test]
fn env_args_cwd() {
    let st = sh("test \"$FOO\" = bar && test \"$1\" = one && test \"$(pwd)\" = /tmp")
        .arg("argv0")
        .arg("one")
        .env("FOO", "bar")
        .current_dir("/tmp")
        .status()
        .unwrap();
    assert!(st.success());
    // env_clear: the child sees only what we pass.
    assert!(sh("test -z \"$HOME\" && test \"$ONLY\" = 1").env_clear().env("ONLY", "1").status().unwrap().success());
}

#[test]
fn stdio_redirect() {
    let path = std::env::temp_dir().join(format!("libsiyana-out-{}", std::process::id()));
    let path = path.to_str().unwrap();
    let out = File::create(path).unwrap();
    let st = sh("echo redirected; echo to-null >&2").stdout(Stdio::Handle(out.handle())).stderr(Stdio::Null).status().unwrap();
    assert!(st.success());
    drop(out);
    assert_eq!(fs::read_to_string(path).unwrap(), "redirected\n");
    fs::remove_file(path).unwrap();
}

#[test]
fn clocks() {
    let t = Instant::now();
    time::sleep(Duration::from_millis(20));
    assert!(t.elapsed() >= Duration::from_millis(20));
    assert!(time::unix_time().as_secs() > 1_700_000_000);
    assert_eq!(process::id(), std::process::id());
}
