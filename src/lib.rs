mod business_logic_layer;
mod data_access_layer;
mod entities;

use std::collections::HashMap;
use std::net::SocketAddr;
use std::time::{Duration, Instant};
pub use crate::entities::Exception;
use crate::data_access_layer::{TypedClientSocket, TypedServerSocket};
use crate::business_logic_layer as bll;
pub use crate::data_access_layer::MAX_DATAGRAM_SIZE;

#[derive(Debug)]
///Events from server
pub enum ServerEvent {
    ///Error on read data from socket
    ExceptionOnRecv(Exception),
    ///Error on write data to socket
    ExceptionOnSend((SocketAddr, Exception)),
}

pub type ContinueRunning = bool;
pub type DisconnectThisClient = bool;

///Game to use with server must implement this trait
pub trait Game {
    /// delta_time: time elapsed from last call
    /// command: ordered commands commands from server
    /// from: Address of command sender
    /// returns tuple with bool value indicating
    /// 1) should server continue running if false stops server
    /// 2) disconnect this client. if true dont send new server state to this client
    fn update(&mut self, delta_time: Duration, commands: Vec<Vec<u8>>, from: SocketAddr) -> (ContinueRunning, DisconnectThisClient);
    ///gets new state to send to client
    /// delta_time: time elapsed throw last call
    /// returns bytes with new game state for client
    /// called once in about 30 milliseconds
    fn draw(&mut self, delta_time: Duration) -> Vec<u8>;
    ///allow client with this IP Address work with server
    /// if false server don't send new state to this client
    /// usually don't implement this method. Use default implementation
    fn allow_connect(&mut self, _from: &SocketAddr) -> bool {
        true
    }
    ///Handles events from server
    /// returns bool value
    /// if returns false stops server
  /// usually don't implement this method. Use default implementation
    fn handle_server_event(&mut self, event: ServerEvent) -> ContinueRunning {
        true
    }
    ///Client to add to recv state from serve
    /// if returns not None then servers on draw sends new state to this client
    /// if client with this IP Address already connected then nothing happens
  /// usually don't implement this method. Use default implementation
    fn add_client(&mut self) -> Option<SocketAddr> {
        None
    }
}

/// Client used to communicate with server. Must be singleton in your app
pub struct ClientSocket {
    socket: TypedClientSocket,
    client: bll::Client,
}

impl ClientSocket {
    ///Create new client and listen on port to recv packets from server_address and send its to them
    pub fn new(port: &str, server_address: &str) -> Result<ClientSocket, Exception> {
        Ok(ClientSocket {
            socket: TypedClientSocket::new(port, server_address)?,
            client: bll::Client::new(),
        })
    }
    ///Send data to server
   /// Don't block current thread
   /// may wait up to 30 milliseconds if you send commands too often
    pub fn send(&mut self, command: Vec<u8>) -> Result<usize, Exception> {
        let command = self.client.send(command);
        self.socket.write(&command)
    }

    ///Reads data fro server
    /// Don't block current thread
    /// Return None if there is no data available
    pub fn recv(&mut self) -> Result<Vec<u8>, Exception> {
        let state = self.socket.read()?;
        let (state, lost) = self.client.recv(state)?;
        for command in lost {
            self.socket.write(&command)?;
        }
        Ok(state)
    }
}

struct ServerSocket {
    socket: TypedServerSocket,
    servers: HashMap<SocketAddr, bll::Server>,
}

impl ServerSocket {
    pub fn new(port: &str) -> Result<ServerSocket, Exception> {
        Ok(ServerSocket { socket: TypedServerSocket::new(port)?, servers: HashMap::new() })
    }

    pub fn recv(&mut self) -> Result<(Vec<Vec<u8>>, SocketAddr), Exception> {
        let (command, from) = self.socket.read()?;
        self.add(&from);
        let command = self.servers.get_mut(&from).unwrap().recv(command)?;
        Ok((command, from))
    }

    pub fn remove(&mut self, client: &SocketAddr) {
        self.servers.remove(&client);
    }

    pub fn contains(&self, client: &SocketAddr) -> bool {
        self.servers.contains_key(client)
    }

    pub fn add(&mut self, client: &SocketAddr) {
        if !self.servers.contains_key(client) {
            self.servers.insert(client.clone(), bll::Server::new());
        }
    }

    pub fn send_to_all(&mut self, state: Vec<u8>) -> Vec<(SocketAddr, Exception)> {
        let mut exceptions = Vec::new();
        for (a, s) in &mut self.servers {
            self.socket.write(a, &s.send(state.clone()))
                .map_err(|e| exceptions.push((a.clone(), e)));
        }
        exceptions
    }
}

const DRAW_PERIOD_IN_MILLIS: u64 = 30;

///Game server to run game
pub struct GameServer<T: Game> {
    game: T,
    socket: ServerSocket,
    is_running: bool,
    draw: bll::WaitTimer,
    update: bll::ElapsedTimer,
    draw_elapsed: bll::ElapsedTimer,
}

impl<T: Game> GameServer<T> {
    ///Crates new server listening port
    pub fn new(game: T, port: &str) -> Result<GameServer<T>, Exception> {
        Ok(GameServer {
            game,
            socket: ServerSocket::new(port)?,
            is_running: true,
            draw: bll::WaitTimer::new(DRAW_PERIOD_IN_MILLIS),
            update: bll::ElapsedTimer::new(),
            draw_elapsed: bll::ElapsedTimer::new(),
        })
    }
    ///Runs game update - draw circle
    /// blocks current thread
    pub fn run(&mut self) {
        while self.is_running {
            self.update();
            self.draw()
        };
    }

    fn draw(&mut self) {
        if self.draw.to_continue() {
            let state = self.game.draw(self.draw_elapsed.elapsed());
            self.game.add_client().map(|a| self.socket.add(&a));
            self.is_running = self.socket.send_to_all(state)
                .into_iter()
                .map(|ex|
                    self.game.handle_server_event(ServerEvent::ExceptionOnSend(ex))
                )
                .all(|b| b);
        }
    }


    fn update(&mut self) {
        self.socket.recv()
            .map(|(commands, from)| {
                if self.game.allow_connect(&from) {
                    let (continue_running, disconnect_this_client) = self.game
                        .update(self.update.elapsed(), commands, from);
                    if disconnect_this_client {
                        self.socket.remove(&from);
                    }
                    self.is_running = continue_running;
                } else {
                    self.socket.remove(&from);
                }
            })
            .map_err(|e|
                self.is_running = self.game
                    .handle_server_event(ServerEvent::ExceptionOnRecv(e))
            );
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
