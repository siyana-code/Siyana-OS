use libsiyana::ipc::{Listener, Stream};
use libsiyana::Error;

#[test]
fn request_reply_and_names() {
    let dir = std::env::temp_dir().join(format!("libsiyana-ipc-{}", std::process::id()));
    unsafe { std::env::set_var("SIYANA_IPC_DIR", &dir) };

    let server = Listener::bind("echo").unwrap();
    assert_eq!(Listener::bind("echo").unwrap_err(), Error::AlreadyExists);

    let client = std::thread::spawn(|| {
        let mut s = Stream::connect("echo").unwrap();
        s.write_all(b"ping").unwrap();
        s.shutdown_write().unwrap();
        let mut reply = Vec::new();
        s.read_to_end(&mut reply).unwrap();
        reply
    });
    let mut conn = server.accept().unwrap();
    let mut req = Vec::new();
    conn.read_to_end(&mut req).unwrap();
    assert_eq!(req, b"ping");
    conn.write_all(b"pong").unwrap();
    drop(conn);
    assert_eq!(client.join().unwrap(), b"pong");

    // Dropping the listener frees the name.
    drop(server);
    assert_eq!(Stream::connect("echo").unwrap_err(), Error::NotFound);
    let again = Listener::bind("echo").unwrap();
    drop(again);

    assert_eq!(Listener::bind("bad/name").unwrap_err(), Error::InvalidInput);
    assert_eq!(Listener::bind("").unwrap_err(), Error::InvalidInput);
    let _ = std::fs::remove_dir_all(dir);
}
