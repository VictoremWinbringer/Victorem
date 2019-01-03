# Victorem
Victorem - easy UDP game server and client framework for simple 2D and 3D games in Rust.
## Example 
### Cargo.toml
```toml
[dependencies]
victorem = "*"
```
### Client 
```rust
use victorem;
fn main() {
    let mut client = victorem::ClientSocket::new("1111", "127.0.0.1:2222").unwrap();
    loop {
        let _ = client
            .send(b"Ping!".to_vec())
            .map_err(|e| println!("{:#?}", e));
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

struct PingPongGame {}

impl victorem::Game for PingPongGame {
    fn handle_command(
        &mut self,
        _delta_time: Duration,
        commands: Vec<Vec<u8>>,
        from: SocketAddr,
    ) -> victorem::ContinueRunning {
        for v in commands {
            if v.len() > 0 {
                println!(
                    "From Client: {} {}",
                    from,
                    String::from_utf8(v).unwrap_or(String::new()),
                );
            }
        }
        true
    }

    fn draw(&mut self, _delta_time: Duration) -> Vec<u8> {
        b"Pong".to_vec()
    }
}

fn main() {
    let mut server = victorem::GameServer::new(PingPongGame {}, "2222").unwrap();
    server.run();
}
```

