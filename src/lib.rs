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
//    pub fn new(port: &str, server_address: &str) -> Result<Client, Box<dyn Error>> {
//        let mut client = crate::business_logic_layer::Client::new(port, server_address)?;
//        let queue = Arc::new(Mutex::new(VecDeque::new()));
//        let moved = queue.clone();
//        thread::spawn(move || {
//            loop {
//                thread::sleep_ms(30);
//                let q = moved.lock()
//                    .ok()
//                    .as_mut()
//                    .and_then(|x|x.pop_front())
//                    .and_then(|x| client.send(x).map_err(|e|error!("On sending {}", e)).ok());
//            }
//        });
//       Ok(Client { client, commandQueue: queue })
//    }

    fn run_process(&self) -> (mpsc::Sender<Vec<u8>>, mpsc::Receiver<Vec<u8>>){
     let (tx1, rx1) = mpsc::channel();
    let (tx2, rx2) = mpsc::channel();
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
