extern crate victorem;

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
