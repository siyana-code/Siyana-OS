use libsiyana::fs::{self, FileType, OpenOptions};
use libsiyana::Error;

fn tmp(name: &str) -> String {
    let d = std::env::temp_dir().join(format!("libsiyana-fs-{}-{name}", std::process::id()));
    let _ = std::fs::remove_dir_all(&d);
    d.to_str().unwrap().to_string()
}

#[test]
fn write_read_append() {
    let dir = tmp("rw");
    fs::create_dir_all(&dir).unwrap();
    let f = format!("{dir}/a.txt");
    fs::write(&f, "hello").unwrap();
    assert_eq!(fs::read_to_string(&f).unwrap(), "hello");
    OpenOptions::new().append(true).open(&f).unwrap().write_all(b" world").unwrap();
    assert_eq!(fs::read(&f).unwrap(), b"hello world");
    fs::write(&f, "x").unwrap(); // create truncates
    assert_eq!(fs::read_to_string(&f).unwrap(), "x");
    fs::remove_dir_all(&dir).unwrap();
    assert!(!fs::exists(&dir));
}

#[test]
fn directories() {
    let dir = tmp("dirs");
    fs::create_dir_all(&format!("{dir}/x/y")).unwrap();
    fs::create_dir_all(&format!("{dir}/x/y")).unwrap(); // existing is fine
    fs::write(&format!("{dir}/b.txt"), "").unwrap();
    fs::write(&format!("{dir}/a.txt"), "").unwrap();
    let names: Vec<_> = fs::read_dir(&dir).unwrap().into_iter().map(|e| (e.name, e.file_type)).collect();
    assert_eq!(
        names,
        [("a.txt".into(), FileType::File), ("b.txt".into(), FileType::File), ("x".into(), FileType::Directory)]
    );
    assert_eq!(fs::create_dir(&dir), Err(Error::AlreadyExists));
    assert_eq!(fs::remove_dir(&format!("{dir}/x")), Err(Error::NotEmpty));
    assert!(fs::exists(&format!("{dir}/x/y")));
    fs::remove_dir_all(&dir).unwrap();
}

#[test]
fn errors() {
    assert_eq!(fs::read("/definitely/not/here").unwrap_err(), Error::NotFound);
    assert_eq!(fs::read_dir("/etc/hostname").unwrap_err(), Error::NotADirectory);
    let dir = tmp("utf8");
    fs::create_dir_all(&dir).unwrap();
    let f = format!("{dir}/bin");
    fs::write(&f, [0xff, 0xfe]).unwrap();
    assert_eq!(fs::read_to_string(&f).unwrap_err(), Error::InvalidUtf8);
    fs::remove_dir_all(&dir).unwrap();
}
