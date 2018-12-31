mod business_logic_layer;
mod data_access_layer;
mod entities;

use crate::entities::*;
use log::error;
use simplelog::LevelFilter;
use std::collections::VecDeque;
use std::error::Error;
use std::sync::mpsc;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time;
use std::net::SocketAddr;
use std::time::Duration;
use crate::entities::{CommandPacket, StatePacket, Exception};
use crate::data_access_layer::{TypedClientSocket, TypedServerSocket};
use crate::business_logic_layer as bll;
use std::sync::atomic::{AtomicBool, Ordering};


#[derive(Debug, Clone, PartialEq)]
pub enum ServerEvent {
    Connected(SocketAddr),
    DisconnectedByTimeout(SocketAddr),
}

pub trait Game {
    fn update(&mut self, delta_time: Duration, command: Vec<u8>, from: SocketAddr) -> bool;
    fn draw(&mut self, delta_time: Duration) -> Vec<u8>;
    fn allow_connect(from: SocketAddr) -> bool {
        true
    }
    fn handle_server_event(event: ServerEvent) {
        eprintln!("Handled {:#?}", event);
    }
}

pub struct ClientSocket {
    socket: TypedClientSocket,
    client: bll::Client,
}

impl ClientSocket {
    pub fn send(&mut self, command: Vec<u8>) -> Result<usize, Exception> {
        let command = self.client.send(command);
        self.socket.write(&command)
    }

    pub fn recv(&mut self) -> Result<Vec<u8>, Exception> {
        let state = self.socket.read()?;
        let (state, lost) = self.client.recv(state)?;
        for command in lost {
            self.socket.write(&command)?;
        }
        Ok(state)
    }
}

pub struct ServerSocket {
    socket: TypedServerSocket,
    server: bll::Server,
}


impl ServerSocket {
    pub fn new(port: &str) -> Result<ServerSocket, Exception> {
        Ok(ServerSocket { socket: TypedServerSocket::new(port)?, server: bll::Server::new() })
    }
    pub fn send(&mut self, state: Vec<u8>, to: &SocketAddr) -> Result<usize, Exception> {
        let state = self.server.send(state);
        self.socket.write(to, &state)
    }

    pub fn recv(&mut self) -> Result<(Vec<Vec<u8>>, SocketAddr), Exception> {
        let (command, from) = self.socket.read()?;
        let command = self.server.recv(command)?;
        Ok((command, from))
    }
}

pub struct GameWrapper<T: Game> {
    game: T,
    socket: ServerSocket,
}

impl<T: Game> GameWrapper<T> {
    fn new(game: T, port: &str) -> Result<GameWrapper<T>, Exception> {
        Ok(GameWrapper {
            game,
            socket: ServerSocket::new(port)?,
        })
    }
}

pub struct GameServer<T: Game> {
    game: T,
    socket: ServerSocket,
}

impl<T: Game> GameServer<T> {
    fn new(game: T, port: &str) -> Result<GameServer<T>, Exception> {
        Ok(GameServer {
            game,
            socket: ServerSocket::new(port)?,
        })
    }

    pub fn run(&mut self) {
        loop {
            //  game.update(Duration::from_millis(1), Vec::new(), S)
            let state = self.game.draw(Duration::from_millis(1));
        }
    }
}


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
