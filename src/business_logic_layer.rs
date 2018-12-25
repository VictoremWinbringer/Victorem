use crate::data_access_layer::{TypedClientSocket, TypedServerSocket};
use crate::entities::{CommandPacket, StatePacket};
use std::error::Error;
use lazy_static::lazy_static;
use std::collections::HashMap;
use std::sync::Mutex;

lazy_static! {
static ref IDS: Mutex<HashMap<String,u64>> = Mutex::new(HashMap::new());
}

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
    key: String,
    client: Client,
}

impl ClientWithId {
    fn new(port: &str, server_address: &str) -> Result<ClientWithId, Box<dyn Error>> {
        let client = Client::new(port, server_address)?;
        let key = port.to_owned() + ":" + server_address;
        IDS.lock()?
            .entry(key)
            .or_insert(1);
        Ok(ClientWithId { key, client })
    }

    fn send(&mut self, command: Vec<u8>) -> Result<usize, Box<dyn Error>> {
        let mut ids = IDS.lock()?;
        let old_id = ids[&self.key];
        let count = self.client.send(old_id, command)?;
        ids.entry(self.key)
            .and_modify(|v| v += 1);
        Ok(count)
    }

    fn recv(&mut self) -> Result<StatePacket, Box<dyn Error>> {
        self.socket.read()
    }
}


struct Server {
    socket: TypedServerSocket
}