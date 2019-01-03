use std::collections::HashMap;
use std::thread;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH, SystemTimeError};

mod version;

mod protocol;

mod id;

pub mod timer;

mod key;

use crate::data_access_layer::Cache;
use crate::entities::{CommandPacket, Exception, StatePacket};
use self::id::{Filter, Generator, Arranger};
use self::version::VersionChecker;
use self::protocol::ProtocolChecker;
use self::timer::SleepTimer;
use self::key as k;

pub struct Client {
    protocol_version: VersionChecker,
    protocol_id: ProtocolChecker,
    id: Generator,
    cache: Cache,
    id_filter: Filter,
    timer: SleepTimer,
    key_generator: k::Generator,
    key_filter: k::Filter,
}

impl Client {
    pub fn new() -> Client {
        let key = k::new_key();
        Client {
            protocol_version: VersionChecker,
            protocol_id: ProtocolChecker,
            id: Generator::new(1),
            cache: Cache::new(),
            id_filter: Filter::new(0),
            timer: SleepTimer::new(30),
            key_filter: k::Filter::new(key),
            key_generator: k::Generator::new(),
        }
    }

    fn create_command(&mut self, command: Vec<u8>) -> CommandPacket {
        CommandPacket {
            protocol_id:  self.protocol_id.get(),
            protocol_version:self.protocol_version.get(),
            id: self.id.generate(),
            command,
            session_key: self.key_generator.generate(),
        }
    }

    pub fn send(&mut self, command: Vec<u8>) -> CommandPacket {
        let command = self.create_command(command);
        self.cache.add(command.clone());
        self.timer.sleep();
        command
    }

    pub fn recv(&mut self, state: StatePacket) -> Result<(Vec<u8>, Vec<CommandPacket>), Exception> {
        self.protocol_version.check(&state)?;
        self.protocol_id.check(&state)?;
        if !self.key_filter.is_valid(&state) {
            self.key_filter = k::Filter::new(state.session_key);
            self.id_filter = Filter::new(0);
        }
        self.id_filter.is_valid_last_recv_id(&state)?;
        let vec = self.cache.get_range(&state.lost_ids);
        Ok((state.state, vec))
    }
}

pub struct Server {
    protocol_version: VersionChecker,
    protocol_id: ProtocolChecker,
    id: Generator,
    arranger: Arranger<CommandPacket>,
    key_generator: k::Generator,
    key_filter: k::Filter,
}

impl Server {
    pub fn new() -> Server {
        let key = k::new_key();
        Server {
            protocol_version: VersionChecker,
            protocol_id: ProtocolChecker,
            id: Generator::new(1),
            arranger: Arranger::new(0),
            key_filter: k::Filter::new(key),
            key_generator: k::Generator::new(),
        }
    }


    pub fn send(&mut self, state: Vec<u8>) -> StatePacket {
        StatePacket {
            protocol_id: self.protocol_id.get(),
            protocol_version: self.protocol_version.get(),
            id: self.id.generate(),
            lost_ids: self.arranger.get_lost(),
            state,
            session_key: self.key_generator.generate(),
        }
    }

    pub fn recv(&mut self, command: CommandPacket) -> Result<Vec<Vec<u8>>, Exception> {
        self.protocol_version.check(&command)?;
        self.protocol_id.check(&command)?;
        if !self.key_filter.is_valid(&command) {
            self.key_filter = k::Filter::new(command.session_key);
            self.arranger = Arranger::new(0);
        }
        self.arranger.add(command)?;
        let vec = self.arranger.arrange();
        Ok(vec.into_iter().map(|v| v.command).collect())
    }
}
