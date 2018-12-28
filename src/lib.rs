mod entities;
mod data_access_layer;
mod business_logic_layer;

use std::collections::VecDeque;
use crate::entities::*;
use std::error::Error;
use std::sync::{Mutex, Arc};
use std::thread;
use log::error;
use std::sync::mpsc;
use std::time;

trait Game {
    fn update(&mut self, delta_time: std::time::Duration, commands: Vec<Vec<u8>>, from_address: &str) -> Vec<u8>;
}

struct GameProxy {
    game: std::sync::Arc<std::sync::Mutex<Game>>
}

impl GameProxy {
    fn new(game: std::sync::Arc<std::sync::Mutex<Game>>) -> GameProxy {
        let mut client = crate::data_access_layer::TypedClientSocket::new("sdsf", "sdfsf").unwrap();
        let mut server = crate::data_access_layer::TypedServerSocket::new("asdfaf").unwrap();
        GameProxy { game }
    }

    fn update(&mut self, delta_time: std::time::Duration, commands: Vec<Vec<u8>>, from_address: &str) -> Vec<u8> {
        let mut game = self.game.lock().unwrap();
        game.update(delta_time, commands, from_address)
    }
}

struct Client {
    commands: mpsc::Sender<Vec<u8>>,
    states: mpsc::Receiver<Vec<u8>>,
}

impl Client {
    pub fn new(port: &str, server_address: &str) -> Result<Client, Box<dyn Error>> {
        let mut client = crate::business_logic_layer::Client::new(port, server_address)?;
       let (tx,rx) =  Client::run_process(client);
       Ok(Client { commands:tx,states:rx })
    }

    fn run_process(mut client: crate::business_logic_layer::Client) -> (mpsc::Sender<Vec<u8>>, mpsc::Receiver<Vec<u8>>) {
        let (tx1, rx1) = mpsc::channel();
        let (tx2, rx2) = mpsc::channel();
        thread::spawn(move || {
            const SEND_TIMEOUT: time::Duration = time::Duration::from_millis(30);
            let mut timer = time::Instant::now();
            loop {
                if timer.elapsed() > SEND_TIMEOUT {
                    timer = time::Instant::now();
                    rx1.try_recv()
                        .and_then(|command| client.send(command))
                        .map_err(|err| error!("{}", err))
                }

                client.recv()
                    .and_then(|state| tx2.send(state))
                    .map_err(|err| error!("{}", err));
            }
        });
        (tx1, rx2)
    }
}

//#[cfg(test)]
//mod tests {
//    #[test]
//    fn it_works() {
//        assert_eq!(1, 1);
//    }
//}
