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
use simplelog::LevelFilter;

//trait Game {
//    fn update(&mut self, delta_time: std::time::Duration, commands: Vec<Vec<u8>>, from_address: &str) -> Vec<u8>;
//}
//
//struct GameProxy {
//    game: std::sync::Arc<std::sync::Mutex<Game>>
//}
//
//impl GameProxy {
//    fn new(game: std::sync::Arc<std::sync::Mutex<Game>>) -> GameProxy {
////        let mut client = crate::data_access_layer::TypedClientSocket::new("sdsf", "sdfsf").unwrap();
////        let mut server = crate::data_access_layer::TypedServerSocket::new("asdfaf").unwrap();
//        GameProxy { game }
//    }
//
//    fn update(&mut self, delta_time: std::time::Duration, commands: Vec<Vec<u8>>, from_address: &str) -> Vec<u8> {
//        let mut game = self.game.lock().unwrap();
//        game.update(delta_time, commands, from_address)
//    }
//}
///// Client used to communicate with server. Must be singleton in your app
//pub struct Client {
//    commands: mpsc::Sender<Vec<u8>>,
//    states: mpsc::Receiver<Vec<u8>>,
//}
//
//impl Client {
//    ///Create new client and listen on port to recv packets from server_address and send its to them
//    pub fn new(port: &str, server_address: &str) -> Result<Client, Exception> {
//        let mut client = crate::business_logic_layer::Client::new(port, server_address)?;
//        crate::data_access_layer::logger::init(LevelFilter::Info)?;
//        let (tx, rx) = Client::run_process(client);
//        Ok(Client { commands: tx, states: rx })
//    }
//
//    fn run_process(mut client: crate::business_logic_layer::Client) -> (mpsc::Sender<Vec<u8>>, mpsc::Receiver<Vec<u8>>) {
//        let (tx1, rx1) = mpsc::channel();
//        let (tx2, rx2) = mpsc::channel();
//        thread::spawn(move || {
//            const SEND_TIMEOUT: time::Duration = time::Duration::from_millis(30);
//            let mut timer = time::Instant::now();
//            loop {
//                if timer.elapsed() > SEND_TIMEOUT {
//                    timer = time::Instant::now();
//                    match rx1.try_recv() {
//                        Ok(b) => client.send(b).map_err(|e| error!("{}", e)),
//                        Err(mpsc::TryRecvError::Disconnected) => break,
//                        Err(e) => Err(error!("{}", e)),
//                    };
//                };
//                client.recv()
//                    .map_err(|e|error!("{}",e))
//                    .and_then(|b| tx2.send(b)
//                        .map_err(|e|error!("{}",e)));
//
//            }
//        });
//        (tx1, rx2)
//    }
//
//    ///Send data to server
//    /// Don't block current thread
//    pub fn send(&self, command: Vec<u8>) {
//        self.commands.send(command).map_err(|e| error!("{}", e));
//    }
//
//    ///Reads data fro server
//    /// Don't block current thread
//    /// Return None if there is no data available
//    pub fn recv(&self) -> Option<Vec<u8>> {
//        self.states.try_recv().ok()
//    }
//}

//#[cfg(test)]
//mod tests {
//    #[test]
//    fn it_works() {
//        assert_eq!(1, 1);
//    }
//}
