extern crate victorem;

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
