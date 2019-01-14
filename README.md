# Victorem
[<img src="https://api.travis-ci.org/VictoremWinbringer/Victorem.svg?branch=master">](https://travis-ci.org/VictoremWinbringer/Victorem)

Easy UDP game server and client framework for creating simple 2D and 3D online game prototype in Rust.
## Example 
### Cargo.toml
```toml
[dependencies]
victorem = "0.8.2"
```
### Client 
```rust
use victorem;
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
```
### Server
```rust
use victorem;
use std::net::SocketAddr;
use std::time::Duration;

struct PingPongGame {
    id: u32,
}

impl victorem::Game for PingPongGame {
    fn handle_command(
        &mut self,
        delta_time: Duration,
        commands: Vec<Vec<u8>>,
        from: SocketAddr,
    ) -> victorem::ContinueRunning {
        for v in commands {
            println!(
                "From Client: {:?} {} {}",
                delta_time,
                from,
                String::from_utf8(v).unwrap_or(String::new()),
            );
        }
        true
    }

    fn draw(&mut self, delta_time: Duration) -> Vec<u8> {
        self.id += 1;
        format!("Pong {} {:?}", self.id, delta_time).into_bytes()
    }
}

fn main() {
    let mut server = victorem::GameServer::new(PingPongGame { id: 0 }, 22222).unwrap();
    server.run();
}
```

