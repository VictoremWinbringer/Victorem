use serde_derive::{Deserialize, Serialize};
use std::error::Error;
use std::fmt::Display;
use std::fmt::Formatter;
use std::io;
use std::time::{Duration, SystemTimeError};

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct CommandPacket {
    pub protocol_id: u8,
    pub protocol_version: u8,
    pub id: u32,
    pub command: Vec<u8>,
    pub session_key: Duration,
}

impl CommandPacket {
    pub fn new(command: Vec<u8>) -> CommandPacket {
        CommandPacket {
            protocol_id: 0,
            protocol_version: 0,
            id: 0,
            command,
            session_key: Duration::default(),
        }
    }
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct StatePacket {
    pub protocol_id: u8,
    pub protocol_version: u8,
    pub id: u32,
    pub lost_ids: Vec<u32>,
    pub state: Vec<u8>,
    pub session_key: Duration,
}

impl StatePacket {
    pub fn new(state: Vec<u8>) -> StatePacket {
        StatePacket {
            protocol_id: 0,
            protocol_version: 0,
            id: 0,
            lost_ids: Vec::new(),
            state,
            session_key: Duration::default(),
        }
    }
}

#[derive(Debug)]
pub enum Exception {
    IoError(io::Error),
    BadProtocolVersion,
    BincodeError(bincode::Error),
    NotOrderedPacketError,
    NotValidIdError,
}

impl Error for Exception {}

impl Display for Exception {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        write!(f, "{:#?}", self)
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
//#[derive(Debug)]
//pub struct LoggerMonad<T>(Result<T, Exception>);
//
//impl<T> LoggerMonad<T> {
//    pub fn new(value: Result<T, Exception>) -> LoggerMonad<T> {
//        LoggerMonad(value)
//    }
//
//    pub fn and_then<U, F: FnOnce(T) -> LoggerMonad<U>>(self, f: F) -> LoggerMonad<U> {
//        match self.0 {
//            Ok(x) => {
//                let monad = f(x);
//                match &monad.0 {
//                    Ok(_) => monad,
//                    Err(e) => {
//                        eprintln!("{:#?}", e);
//                        monad
//                    }
//                }
//            }
//            Err(e) => LoggerMonad::new(Err(e)),
//        }
//    }
//
//    pub fn map<U, F: FnOnce(T) -> U>(self, f: F) -> LoggerMonad<U> {
//        self.and_then(|x| LoggerMonad::new(Ok(f(x))))
//    }
//
//    pub fn and<U>(self, data: LoggerMonad<U>) -> LoggerMonad<U> {
//        self.and_then(|_| data)
//    }
//
//    pub fn unwrap(self) -> T {
//        self.0.unwrap()
//    }
//
//    pub fn unwrap_or(self, def: T) -> T {
//        self.0.unwrap_or(def)
//    }
//}
