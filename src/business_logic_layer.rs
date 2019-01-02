use crate::data_access_layer::Cache;
use crate::entities::{CommandPacket, Exception, StatePacket};
use std::collections::HashMap;
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

const MAX_SAVED: usize = 2000;

impl<T: IWithId> Arranger<T> {
    fn clear_if_overflows(&mut self) {
        use itertools::*;

        if self.packets.len() > MAX_SAVED {
            self.packets = self.packets.
                drain()
                .sorted_by(|a, b| Ord::cmp(&a.0, &b.0))
                .skip(MAX_SAVED / 2)
                .collect();
            let max_id = self.packets.iter().max_by(|x, y| x.0.cmp(y.0))
                .map(|x| x.0.clone());
            max_id.map(|x| {
                let id = if x > 0 {
                    x - 1
                } else { 0 };
                self.filter.set(id)
            });
        }
    }

    pub fn add(&mut self, data: T) -> Result<(), Exception> {
        self.clear_if_overflows();
        self.filter.is_valid_last_recv_id(&data)?;
        self.packets.entry(data.get()).or_insert(data);
        Ok(())
    }

    fn set_last_valid(&mut self, packets: &Vec<T>) {
        packets
            .iter()
            .map(|p| p.get())
            .max()
            .map(|max| self.filter.set(max));
    }

    fn get_lost(&self) -> Vec<u32> {
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

    fn get_valid(&mut self) -> Vec<T> {
        let mut i = self.filter.get() + 1;
        let mut vec: Vec<T> = Vec::new();
        while self.packets.contains_key(&i) {
            vec.push(self.packets.remove(&i).unwrap());
            i += 1;
        }
        vec
    }
}

struct SleepTimer {
    time: Duration,
    instant: Instant,
}

impl SleepTimer {
    fn sleep(&mut self) {
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

pub struct WaitTimer {
    time: Duration,
    instant: Instant,
}

impl WaitTimer {
    pub fn new(millis: u64) -> WaitTimer {
        WaitTimer {
            time: Duration::from_millis(millis),
            instant: Instant::now(),
        }
    }
    pub fn to_continue(&mut self) -> bool {
        if self.instant.elapsed() > self.time {
            self.instant = Instant::now();
            true
        } else {
            false
        }
    }
}

pub struct ElapsedTimer {
    time: Duration,
    instant: Instant,
}

impl ElapsedTimer {
    pub fn new() -> ElapsedTimer {
        ElapsedTimer {
            time: Duration::new(0, 0),
            instant: Instant::now(),
        }
    }
    pub fn elapsed(&mut self) -> Duration {
        let elapsed = self.instant.elapsed();
        let res = elapsed - self.time;
        self.time = elapsed;
        res
    }
}

pub struct Client {
    version: VersionChecker,
    protocol: ProtocolChecker,
    id: Generator,
    cache: Cache,
    filter: Filter,
    timer: SleepTimer,
}

impl Client {
    pub fn new() -> Client {
        Client {
            version: VersionChecker,
            protocol: ProtocolChecker,
            id: Generator { id: 1 },
            cache: Cache::new(),
            filter: Filter { id: 0 },
            timer: SleepTimer {
                time: Duration::from_millis(30),
                instant: Instant::now(),
            },
        }
    }
    pub fn send(&mut self, command: Vec<u8>) -> CommandPacket {
        let mut command = CommandPacket::new(command);
        let cr = &mut command;
        self.version.set(cr);
        self.protocol.set(cr);
        self.id.set(cr);
        self.timer.sleep();
        self.cache.add(command.clone());
        command
    }

    pub fn recv(&mut self, state: StatePacket) -> Result<(Vec<u8>, Vec<CommandPacket>), Exception> {
        self.version.check(&state)?;
        self.protocol.check(&state)?;
        self.filter.filter_and_set_last_recv_id(&state)?;
        let vec = self.cache.get_range(&state.lost_ids);
        Ok((state.state, vec))
    }
}

pub struct Server {
    version: VersionChecker,
    protocol: ProtocolChecker,
    id: Generator,
    arranger: Arranger<CommandPacket>,
}

impl Server {
    pub fn new() -> Server {
        Server {
            version: VersionChecker,
            protocol: ProtocolChecker,
            id: Generator { id: 1 },
            arranger: Arranger {
                filter: Filter { id: 0 },
                packets: HashMap::new(),
            },
        }
    }

    pub fn send(&mut self, state: Vec<u8>) -> StatePacket {
        let mut state = StatePacket::new(state);
        let sr = &mut state;
        self.version.set(sr);
        self.protocol.set(sr);
        self.id.set(sr);
        state.lost_ids = self.arranger.get_lost();
        state
    }

    pub fn recv(&mut self, command: CommandPacket) -> Result<Vec<Vec<u8>>, Exception> {
        self.version.check(&command)?;
        self.protocol.check(&command)?;
        self.arranger.add(command)?;
        let vec = self.arranger.get_valid();
        self.arranger.set_last_valid(&vec);
        Ok(vec.into_iter().map(|v| v.command).collect())
    }
}
