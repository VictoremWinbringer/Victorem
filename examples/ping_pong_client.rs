extern crate victorem;

use std::time::{Duration, Instant};

fn main() {
    let mut client = victorem::ClientSocket::new(11111, "127.0.0.1:22222").unwrap();
    let mut id: u32 = 0;
    let mut timer = Instant::now();
    let period = Duration::from_millis(100);
    loop {
        if timer.elapsed() > period {
            timer = Instant::now();
            id += 1;
            let _ = client.send(format!("Ping {}", id).into_bytes());
        }
        let _ = client
            .recv()
            .map(|v| String::from_utf8(v).unwrap_or(String::new()))
            .map(|s| println!("From Server: {}", s));
    }
}
