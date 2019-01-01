extern crate victorem;

use std::time::Duration;
use std::net::SocketAddr;

struct ChatGame {}

impl victorem::Game for ChatGame {
    fn update(&mut self, delta_time: Duration, commands: Vec<Vec<u8>>, from: SocketAddr) -> (victorem::ContinueRunning, victorem::DisconnectThisClient) {
        for v in commands {
            if v.len() > 0 {
                println!("From Client: {}", String::from_utf8(v).unwrap_or(String::new()));
            }
        }
        (true, false)
    }

    fn draw(&mut self, delta_time: Duration) -> Vec<u8> {
        b"Pong".to_vec()
    }
}

fn main() {
    let mut server = victorem::GameServer::new(ChatGame {}, "2222").unwrap();
    server.run();
}