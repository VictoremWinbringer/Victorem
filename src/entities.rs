use serde_derive::{Serialize, Deserialize};
use std::convert::From;
use std::error::Error;
use std::fmt::Display;
use std::fmt::Formatter;

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct CommandPacket {
    pub protocol_id: u8,
    pub protocol_version: u8,
    pub  id: u64,
    pub  command: Vec<u8>,
}

impl CommandPacket {
    fn new(command: Vec<u8>) -> CommandPacket {
        CommandPacket { protocol_id: 0, protocol_version: 0, id: 0, command }
    }
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct StatePacket {
    pub protocol_id: u8,
    pub protocol_version: u8,
    pub  id: u64,
    pub lost_ids: Vec<u64>,
    pub state: Vec<u8>,
}

impl StatePacket {
    fn new(state:Vec<u8>)->StatePacket{
        StatePacket{protocol_id:0,protocol_version:0,id:0,lost_ids:Vec::new(),state}
    }
}

#[derive(Debug)]
pub enum Exception {
    IoError(std::io::Error),
    BadProtocolVersion,
    BincodeError(bincode::Error),
    SetLoggerError(log::SetLoggerError),
    NotOrderedPacketError,
}

impl Error for Exception {

}

impl Display for Exception {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        write!(f,"{:#?}",self)
    }
}

impl std::convert::From<std::io::Error> for Exception {
    fn from(err: std::io::Error) -> Self {
        Exception::IoError(err)
    }
}

impl std::convert::From<bincode::Error> for Exception {
    fn from(err: bincode::Error) -> Self {
        Exception::BincodeError(err)
    }
}

impl std::convert::From<log::SetLoggerError> for Exception {
    fn from(err: log::SetLoggerError) -> Self {
        Exception::SetLoggerError(err)
    }
}