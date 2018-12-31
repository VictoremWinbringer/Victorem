use crate::entities::{CommandPacket, Exception, StatePacket};
//TODO: disconnect after 10 seconds and que to send one packet in 30 ms and send lost ids
use crate::data_access_layer::Cache;
use std::collections::{HashMap, VecDeque};
use std::thread;
use std::time::{Duration, Instant};

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

struct VersionChecker;

impl VersionChecker {
    fn check(&self, data: &impl IWithVersion) -> Result<(), Exception> {
        if data.get() == PROTOCOL_VERSION {
            Ok(())
        } else {
            Err(Exception::BadProtocolVersion)
        }
    }

    fn set(&self, data: &mut impl IWithVersion) {
        data.set(PROTOCOL_VERSION)
    }
}

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

struct ProtocolChecker;

impl ProtocolChecker {
    fn check(&self, data: &impl IWithProtocol) -> Result<(), Exception> {
        if data.get() == PROTOCOL_ID {
            Ok(())
        } else {
            Err(Exception::BadProtocolVersion)
        }
    }

    pub fn set(&self, data: &mut impl IWithProtocol) {
        data.set(PROTOCOL_ID)
    }
}

pub trait IWithId {
    fn get(&self) -> u32;
    fn set(&mut self, id: u32);
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

struct Generator {
    id: u32,
}

impl Generator {
    fn set(&mut self, data: &mut impl IWithId) {
        data.set(self.id);
        self.id += 1;
    }
}

pub struct Filter {
    id: u32,
}

impl Filter {
    pub fn is_valid_last_recv_id(&self, data: &impl IWithId) -> Result<(), Exception> {
        if data.get() > self.id {
            Ok(())
        } else {
            Err(Exception::NotValidIdError)
        }
    }
    pub fn set(&mut self, id: u32) {
        self.id = id
    }

    pub fn get(&self) -> u32 {
        self.id
    }

    pub fn filter_and_set_last_recv_id(&mut self, data: &impl IWithId) -> Result<(), Exception> {
        self.is_valid_last_recv_id(data)?;
        self.set(data.get());
        Ok(())
    }
}

struct Arranger<T: IWithId> {
    filter: Filter,
    packets: HashMap<u32, T>,
}

impl<T: IWithId> Arranger<T> {
    pub fn add(&mut self, data: T) -> Result<(), Exception> {
        self.filter.is_valid_last_recv_id(&data)?;
        self.packets.entry(data.get()).or_insert(data);
        Ok(())
    }

    pub fn get_valid_and_lost(&mut self) -> (Vec<T>, Vec<u32>) {
        let packets = self.find_valid();
        let lost_ids = self.find_lost();
        self.set_last_valid(&packets);
        (packets, lost_ids)
    }

    fn set_last_valid(&mut self, packets: &Vec<T>) {
        packets
            .iter()
            .map(|p| p.get())
            .max()
            .map(|max| self.filter.set(max));
    }

    fn find_lost(&self) -> Vec<u32> {
        self.packets
            .keys()
            .max()
            .map(|max| (self.filter.get() + 1, max.clone()))
            .map(|(min, max)| min..=max)
            .map(|range| range.filter(|i| !self.packets.contains_key(i)))
            .map(|filter| {
                let ids: Vec<u32> = filter.collect();
                ids
            })
            .unwrap_or(Vec::new())
    }

    fn find_valid(&mut self) -> Vec<T> {
        let mut i = self.filter.get() + 1;
        let mut vec: Vec<T> = Vec::new();
        while self.packets.contains_key(&i) {
            vec.push(self.packets.remove(&i).unwrap());
            i += 1;
        }
        vec
    }
}

struct Timer {
    time: Duration,
    instant: Instant,
}

impl Timer {
    fn wait(&mut self) {
        let elapsed = self.instant.elapsed();
        self.time
            .checked_sub(elapsed)
            .and_then(|d| {
                if d == Duration::new(0, 0) {
                    None
                } else {
                    Some(d)
                }
            })
            .map(|d| thread::sleep(d));
        self.instant = Instant::now();
    }
}

pub struct Client {
    //    id: u32,
    //    socket: TypedClientSocket,
    //    last_recv_id: u32,
    //    send_packets: HashMap<u32, CommandPacket>,
    version: VersionChecker,
    protocol: ProtocolChecker,
    id: Generator,
    cache: Cache,
    filter: Filter,
    timer: Timer,
}

impl Client {
    pub fn send(&mut self, command: Vec<u8>) -> CommandPacket {
        let mut command = CommandPacket::new(command);
        let cr = &mut command;
        self.version.set(cr);
        self.protocol.set(cr);
        self.id.set(cr);
        self.timer.wait();
        self.cache.add(command.clone());
        command
    }

    pub fn recv(&mut self, state: StatePacket) -> Result<(Vec<u8>, Vec<CommandPacket>), Exception> {
        self.version.check(&state)?;
        self.protocol.check(&state)?;
        self.filter.filter_and_set_last_recv_id(&state)?;
        let mut vec = Vec::<CommandPacket>::new();
        for id in state.lost_ids {
            self.cache.get(id).map(|p| vec.push(p));
        }
        Ok((state.state, vec))
    }


    //    fn recv_and_resend_lost_command(&mut self) -> Result<StatePacket, Exception> {
    //        let packet = self.recv_ordered()?;
    //        for id in packet.lost_ids.iter() {
    //            self.send_packets.get(id)
    //                .and_then(|p| self.socket.write(p)
    //                    .map_err(|e| error!("on resend packet {:?}", e))
    //                    .ok());
    //        }
    //        Ok(packet)
    //    }
    //
    //    pub fn recv(&mut self) -> Result<Vec<u8>, Exception> {
    //        let packet = self.recv_and_resend_lost_command()?;
    //        Ok(packet.state)
    //    }
}

struct Server {
    // socket: TypedServerSocket
}

impl Server {
    fn send(&mut self, state: Vec<u8>) -> StatePacket {
        unimplemented!()
    }
    fn recv(&mut self, command: CommandPacket) -> Vec<u8> {
        unimplemented!()
    }
}
