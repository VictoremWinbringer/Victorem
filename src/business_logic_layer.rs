use crate::data_access_layer::{TypedClientSocket, TypedServerSocket};
use crate::entities::{CommandPacket, StatePacket};
use std::error::Error;
use lazy_static::lazy_static;
use std::collections::HashMap;
use std::sync::Mutex;

struct Client {
    socket: TypedClientSocket
}

impl Client {
    fn new(port: &str, server_address: &str) -> Result<Client, Box<dyn Error>> {
        let socket = TypedClientSocket::new(port, server_address)?;
        Ok(Client { socket })
    }

    fn send(&mut self, id: u64, command: Vec<u8>) -> Result<usize, Box<dyn Error>> {
        self.socket.write(&CommandPacket::new(id, command))
    }

    fn recv(&mut self) -> Result<StatePacket, Box<dyn Error>> {
        self.socket.read()
    }
}

struct ClientWithId {
    id:u64,
    client: Client,
}

impl ClientWithId {
    fn new(port: &str, server_address: &str) -> Result<ClientWithId, Box<dyn Error>> {
        let client = Client::new(port, server_address)?;;
        Ok(ClientWithId { id:1, client })
    }

    fn send(&mut self, command: Vec<u8>) -> Result<usize, Box<dyn Error>> {
        let id = self.id.clone();
        self.id +=1;
       self.client.send(id, command)
    }

    fn recv(&mut self) -> Result<StatePacket, Box<dyn Error>> {
        self.client.recv()
    }
}


struct Server {
    socket: TypedServerSocket
}