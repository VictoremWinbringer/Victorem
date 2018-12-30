use crate::data_access_layer::{TypedClientSocket, TypedServerSocket};
use crate::entities::{CommandPacket, StatePacket, Exception};
use std::error::Error;
use lazy_static::lazy_static;
use std::collections::HashMap;
use std::sync::Mutex;
use std::fmt::Display;
use std::fmt::Formatter;
use std::any::Any;
use log::error;
//TODO: disconnect after 10 seconds and que to send one packet in 30 ms and send lost ids

mod versions {
    use crate::entities::{StatePacket, CommandPacket, Exception};

    const PROTOCOL_VERSION: u8 = 1;

    trait IWithVersion {
        fn get(&self) -> u8;
        fn set(&mut self, version: u8);
    }

    impl IWithVersion for StatePacket {
        fn get(&self) -> u8 {
            self.protocol_version
        }

        fn set(&mut self, version: u8) {
            self.protocol_version = version
        }
    }

    impl IWithVersion for CommandPacket {
        fn get(&self) -> u8 {
            self.protocol_version
        }

        fn set(&mut self, version: u8) {
            self.protocol_version = version
        }
    }

    fn check(data: &impl IWithVersion) -> Result<(), Exception> {
        if data.get() == PROTOCOL_VERSION {
            Ok(())
        } else {
            Err(Exception::BadProtocolVersion)
        }
    }

    fn set(data: &mut impl IWithVersion) {
        data.set(PROTOCOL_VERSION)
    }
}

mod protocol_id {
    use crate::entities::{StatePacket, CommandPacket, Exception};

    const PROTOCOL_ID: u8 = 8;

   pub trait IWithProtocol {
        fn get(&self) -> u8;
        fn set(&mut self, id: u8);
    }

    impl IWithProtocol for StatePacket {
        fn get(&self) -> u8 {
            self.protocol_id
        }

        fn set(&mut self, id: u8) {
            self.protocol_id = id
        }
    }

    impl IWithProtocol for CommandPacket {
        fn get(&self) -> u8 {
            self.protocol_id
        }

        fn set(&mut self, id: u8) {
            self.protocol_id = id
        }
    }

 pub   fn check(data: &impl IWithProtocol) -> Result<(), Exception> {
        if data.get() == PROTOCOL_ID {
            Ok(())
        } else {
            Err(Exception::BadProtocolVersion)
        }
    }

   pub fn set(data: &mut impl IWithProtocol){
        data.set(PROTOCOL_ID)
    }
}

mod id {
    use crate::entities::{StatePacket, CommandPacket, Exception};

    trait IWithId {
        fn get(&self) -> u32;
        fn set(&mut self, id:u32);
    }

    impl IWithId for StatePacket {
        fn get(&self) -> u32 {
            self.id
        }

        fn set(&mut self, id: u32) {
            self.id = id
        }
    }

    impl IWithId for CommandPacket {
        fn get(&self) -> u32 {
            self.id
        }

        fn set(&mut self, id: u32) {
            self.id = id
        }
    }

    struct IdGenerator{
        id:u32
    }

    impl IdGenerator{
        fn set(&mut self, data:&mut impl IWithId){
            data.set(self.id)
        }
    }
}

pub struct Client {
    id: u32,
    socket: TypedClientSocket,
    last_recv_id: u32,
    send_packets: HashMap<u32, CommandPacket>,
}

impl Client {
    const MAX_SAVED_PACKETS: usize = 6000;

    pub fn new(port: &str, server_address: &str) -> Result<Client, Exception> {
        let socket = TypedClientSocket::new(port, server_address)?;
        Ok(Client { id: 1, socket, last_recv_id: 0, send_packets: HashMap::new() })
    }

    fn write(&mut self, command: CommandPacket) -> Result<usize, Exception> {
        self.socket.write(&command)
    }

    fn read(&mut self) -> Result<StatePacket, Exception> {
        self.socket.read()
    }

    fn recv_ordered(&mut self) -> Result<StatePacket, Exception> {

        let packet = self.read()?;
        self::protocol_id::check(&packet)?;
        if packet.id <= self.id {
            Err(Exception::NotOrderedPacketError)
        } else {
            self.id = packet.id;
            Ok(packet)
        }
    }

    fn recv_and_resend_lost_command(&mut self) -> Result<StatePacket, Exception> {
        let packet = self.recv_ordered()?;
        for id in packet.lost_ids.iter() {
            self.send_packets.get(id)
                .and_then(|p| self.socket.write(p)
                    .map_err(|e| error!("on resend packet {:?}", e))
                    .ok());
        }
        Ok(packet)
    }

    pub fn recv(&mut self) -> Result<Vec<u8>, Exception> {
        let packet = self.recv_and_resend_lost_command()?;
        Ok(packet.state)
    }

    //////////////////////////////////////////////////////////////////////////////////////////////////////////////
    fn send_and_remember(&mut self, command: CommandPacket) -> Result<usize, Exception> {
        if self.send_packets.len() > Client::MAX_SAVED_PACKETS {
            self.send_packets = self.send_packets.clone()
                .into_iter()
                .skip(Client::MAX_SAVED_PACKETS / 2)
                .collect();
        }
        self.send_packets.entry(command.id).or_insert(command.clone());
        self.write(command)
    }

    fn send_with_id(&mut self, id: u32, command: Vec<u8>) -> Result<usize, Exception> {
        self.send_and_remember(CommandPacket {
            protocol_id: 0,
            protocol_version: 0,
            id: id,
            command: command,
        })
    }

    fn send_and_increase_last_send_id(&mut self, command: Vec<u8>) -> Result<usize, Exception> {
        let id = self.id.clone();
        self.id += 1;
        self.send_with_id(id, command)
    }

    pub fn send(&mut self, command: Vec<u8>) -> Result<usize, Exception> {
        self.send_and_increase_last_send_id(command)
    }
}


struct Server {
    socket: TypedServerSocket
}

impl Server {}
