//! The smallest portable Siyana app.
//!
//!   cargo run --example hello -- some args
//!   cargo check --target x86_64-unknown-none --example hello   # Siyana build

#![cfg_attr(not(target_os = "linux"), no_std, no_main)]

use libsiyana::prelude::*;
use libsiyana::{env, process, time};

libsiyana::entry!(app_main);

fn app_main() -> i32 {
    let start = time::Instant::now();
    let args = env::args();
    println!("hello from pid {}", process::id());
    println!("args: {:?}", &args[1..]);
    println!("USER = {}", env::var("USER").unwrap_or_else(|| "?".to_string()));
    println!("unix time: {} s", time::unix_time().as_secs());
    time::sleep(Duration::from_millis(50));
    println!("took {:?}", start.elapsed());
    0
}
