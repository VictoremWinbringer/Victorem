use crate::data_access_layer::{TypedClientSocket, TypedServerSocket};
use crate::entities::{CommandPacket, StatePacket};
use std::error::Error;
use lazy_static::lazy_static;
use std::collections::HashMap;
use std::sync::Mutex;
use std::fmt::Display;
use std::fmt::Formatter;
use std::any::Any;
use log::error;
//TODO: disconnect after 10 seconds and que to send one packet in 30 ms and send lost ids

#[derive(Debug)]
struct NotOrderedPacketError(String);

impl Error for NotOrderedPacketError {}

impl Display for NotOrderedPacketError {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

pub struct Client {
    id: u64,
    socket: TypedClientSocket,
    last_recv_id: u64,
    send_packets: HashMap<u64, CommandPacket>,
}

impl Client {
    const MAX_SAVED_PACKETS: usize = 6000;

    pub fn new(port: &str, server_address: &str) -> Result<Client, Box<dyn Error>> {
        let socket = TypedClientSocket::new(port, server_address)?;
        Ok(Client { id: 1, socket, last_recv_id: 0, send_packets: HashMap::new() })
    }

    fn write(&mut self, command: CommandPacket) -> Result<usize, Box<dyn Error>> {
        self.socket.write(&command)
    }

    fn read(&mut self) -> Result<StatePacket, Box<dyn Error>> {
        self.socket.read()
    }

    fn recv_ordered(&mut self) -> Result<StatePacket, Box<dyn Error>> {
        let packet = self.read()?;
        if packet.id <= self.id {
            Err(Box::new(NotOrderedPacketError("Not ordered packet".into())))
        } else {
            self.id = packet.id;
            Ok(packet)
        }
    }

    fn recv_and_resend_lost_command(&mut self) -> Result<StatePacket, Box<dyn Error>> {
        let packet = self.recv_ordered()?;
        for id in packet.lost_ids.iter() {
         self.send_packets.get(id)
                .and_then(|p| self.socket.write(p)
                    .map_err(|e| error!("on resend packet {}", e))
                    .ok());
        }
//        for p in self.send_packets.iter() {
//            if packet.lost_ids.contains(&p.id) {
//                self.socket.write(p);
//            }
//        }
        Ok(packet)
    }

    pub fn recv(&mut self) -> Result<Vec<u8>, Box<dyn Error>> {
        let packet = self.recv_and_resend_lost_command()?;
       Ok(packet.state)
    }

    //////////////////////////////////////////////////////////////////////////////////////////////////////////////
    fn send_and_remember(&mut self, command: CommandPacket) -> Result<usize, Box<dyn Error>> {
        if self.send_packets.len() > Client::MAX_SAVED_PACKETS {
            self.send_packets = self.send_packets.clone()
                .into_iter()
                .skip(Client::MAX_SAVED_PACKETS / 2)
                .collect();
        }
        self.send_packets.entry(command.id).or_insert(command.clone());
        self.write(command)
    }

    fn send_with_id(&mut self, id: u64, command: Vec<u8>) -> Result<usize, Box<dyn Error>> {
        self.send_and_remember(CommandPacket::new(id, command))
    }

    fn send_and_increase_last_send_id(&mut self, command: Vec<u8>) -> Result<usize, Box<dyn Error>> {
        let id = self.id.clone();
        self.id += 1;
        self.send_with_id(id, command)
    }

    pub fn send(&mut self, command: Vec<u8>) -> Result<usize, Box<dyn Error>> {
        self.send_and_increase_last_send_id(command)
    }
}

struct Server {
    socket: TypedServerSocket
}
