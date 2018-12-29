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

trait Middleware<T: ?Sized> {
    fn execute<'a: 'b, 'b>(&mut self, data: &'a mut T) -> Result<(), Exception>;
    fn next(&mut self) -> &mut Option<Box<dyn Middleware<T>>>;
    fn run<'a: 'b, 'b>(&mut self, data: &'a mut T) -> Result<(), Exception> {
        self.execute(data)?;
        match &mut self.next() {
            Some(next) => next.execute(data),
            None => Ok(()),
        }
    }
}

const PROTOCOL_ID: u8 = 8;
const PROTOCOL_VERSION: u8 = 1;

struct ProtocolVersionChecker<T> {
    next: Option<Box<Middleware<T>>>,
}

impl Middleware<StatePacket> for ProtocolVersionChecker<StatePacket> {
    fn execute<'a: 'b, 'b>(&mut self, data: &'a mut StatePacket) -> Result<(), Exception> {
        if data.protocol_version == PROTOCOL_VERSION {
            Ok(())
        } else {
            Err(Exception::BadProtocolVersion)
        }
    }

    fn next(&mut self) -> &mut Option<Box<Middleware<StatePacket>>> {
        &mut self.next
    }
}

impl Middleware<CommandPacket> for ProtocolVersionChecker<CommandPacket> {
    fn execute<'a: 'b, 'b>(&mut self, data: &'a mut CommandPacket) -> Result<(), Exception> {
        if data.protocol_version == PROTOCOL_VERSION {
            Ok(())
        } else {
            Err(Exception::BadProtocolVersion)
        }
    }

    fn next(&mut self) -> &mut Option<Box<Middleware<CommandPacket>>> {
        &mut self.next
    }
}

struct ProtocolVersionSetter<T> {
    next: Option<Box<Middleware<T>>>,
}

impl Middleware<CommandPacket> for ProtocolVersionSetter<CommandPacket> {
    fn execute<'a: 'b, 'b>(&mut self, data: &'a mut CommandPacket) -> Result<(), Exception> {
        data.protocol_version = PROTOCOL_VERSION;
        Ok(())
    }

    fn next(&mut self) -> &mut Option<Box<Middleware<CommandPacket>>> { &mut self.next }
}

impl Middleware<StatePacket> for ProtocolVersionSetter<StatePacket> {
    fn execute<'a: 'b, 'b>(&mut self, data: &'a mut StatePacket) -> Result<(), Exception> {
        data.protocol_version = PROTOCOL_VERSION;
        Ok(())
    }

    fn next(&mut self) -> &mut Option<Box<Middleware<StatePacket>>> {
        &mut self.next
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

    fn send_with_id(&mut self, id: u64, command: Vec<u8>) -> Result<usize, Exception> {
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
