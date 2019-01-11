mod version;

mod protocol;

mod id;

pub mod timer;

mod key;

use self::id::{Arranger, Filter, Generator};
use self::key as k;
use self::protocol::ProtocolChecker;
use self::timer::SleepTimer;
use self::version::VersionChecker;
use crate::data_access_layer::Cache;
use crate::entities::{CommandPacket, Exception, StatePacket};

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
            protocol_id: self.protocol_id.get(),
            protocol_version: self.protocol_version.get(),
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
        self.id_filter.filter(&state)?;
        let vec = self.get_lost(state.last_received, state.sequence);
        Ok((state.state, vec))
    }

    fn get_lost(&mut self, max_id: u32, sequence: u32) -> Vec<CommandPacket> {
        if max_id == 0 || sequence == 0 {
            return Vec::new();
        }

        let mut x = max_id;
        let mut y = 0;
        let mut ids = Vec::<u32>::new();
        while x > 0 && y < 32 {
            x -= 1;
            let mask = 1u32 << y;
            y += 1;
            let res = sequence & mask;
            if res > 0 {
                ids.push(x);
            }
        }
        self.cache.get_range(&ids)
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
        let (sequence, last_id) = self.arranger.get_lost();
        StatePacket {
            protocol_id: self.protocol_id.get(),
            protocol_version: self.protocol_version.get(),
            id: self.id.generate(),
            state: state,
            last_received: last_id,
            sequence: sequence,
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

#[cfg(test)]
mod bll_test {
    use crate::business_logic_layer::{Client, Server};

    #[test]
    fn client_should_recv_ordered_packets() {
        let mut client = Client::new();
        let mut server = Server::new();
        let packet1 = server.send(vec![1]);
        let packet2 = server.send(vec![2]);
        let packet3 = server.send(vec![3]);
        let r_packet1 = client.recv(packet1);
        let r_packet3 = client.recv(packet3);
        let r_packet2 = client.recv(packet2);
        assert!({
            match r_packet1 {
                Err(_) => false,
                Ok((v, _)) => v == vec![1]
            }
        });

        assert!({
            match r_packet3 {
                Err(_) => false,
                Ok((v, _)) => v == vec![3]
            }
        });

        assert!({
            match r_packet2 {
                Err(crate::entities::Exception::NotOrderedPacketError) => true,
                _ => false,
            }
        });
    }

    #[test]
    fn server_should_recv_ordered_and_reliable_packets() {
        let mut client = Client::new();
        let mut server = Server::new();
        let command1 = client.send(vec![1]);
        let command2 = client.send(vec![2]);
        let command3 = client.send(vec![3]);
        assert_eq!(3, command3.id);
        let r1 = server.recv(command1);
        let r3 = server.recv(command3);
        assert!({
            match r1 {
                Err(_) => false,
                Ok(v) => v.contains(&vec![1])
            }
        });
        assert!({
            match r3 {
                Err(_) => false,
                Ok(v) => v.is_empty(),
            }
        });
        let state1 = server.send(vec![4]);
        assert_eq!(3, state1.last_received);
        assert_eq!(1, state1.sequence);
        let (b, commands) = client.recv(state1).unwrap();

        let r2 = server.recv(commands.first().unwrap().to_owned());
        assert!({
            match r2 {
                Err(_) => false,
                Ok(v) => v.len() == 2 && v.contains(&vec![2]) && v.contains(&vec![3]) && v[0] == vec![2]
            }
        });
    }
}