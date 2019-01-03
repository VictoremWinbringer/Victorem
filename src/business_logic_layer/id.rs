use crate::entities::{CommandPacket, Exception, StatePacket};
use std::collections::HashMap;

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

pub struct Generator {
    id: u32,
}

impl Generator {
    pub fn new(start: u32) -> Generator { Generator { id: start } }
    pub fn generate(&mut self) -> u32 {
        let result = self.id;
        self.id += 1;
        result
    }
}

pub struct Filter {
    id: u32,
}

impl Filter {
    pub fn new(start: u32) -> Filter { Filter { id: start } }

    pub fn is_valid_last_recv_id(&mut self, data: &impl IWithId) -> Result<(), Exception> {
        if data.get() > self.id
            || self.id - data.get() > MAX_ID_BREAK {
            self.id = data.get();
            Ok(())
        } else {
            Err(Exception::NotValidIdError)
        }
    }
}

pub struct Arranger<T: IWithId> {
    last_id: u32,
    packets: HashMap<u32, T>,
    received: Vec<u32>,
}


const MAX_ID_BREAK: u32 = 64;
const MAX_SAVED: usize = (MAX_ID_BREAK * 2) as usize;
const MAX_RECEIVED: usize = MAX_SAVED;

impl<T: IWithId> Arranger<T> {
    pub fn new(last_id: u32) -> Arranger<T> {
        Arranger {
            last_id,
            packets: HashMap::new(),
            received: Vec::new(),
        }
    }

    pub fn arrange(&mut self) -> Vec<T> {
        let vec = self.get_valid();
        self.set_last_valid(&vec);
        vec
    }

    pub fn add(&mut self, data: T) -> Result<(), Exception> {
        let id = data.get();
        if id + MAX_ID_BREAK < self.last_id || self.last_id + MAX_ID_BREAK < id {
            self.last_id = id;
            self.received = Vec::new()
        }
        self.clear_if_overflows();
        if self.received.contains(&data.get()) {
            Err(Exception::NotValidIdError)
        } else {
            self.received.push(data.get());
            self.packets.entry(data.get()).or_insert(data);
            Ok(())
        }
    }

    fn clear_if_overflows(&mut self) {
        use itertools::*;

        if self.packets.len() > MAX_SAVED {
            let min_id = self
                .packets
                .iter()
                .min_by(|x, y| x.0.cmp(y.0))
                .map(|x| *x.0);
            min_id.map(|x| {
                self.last_id = if x > 0 { x - 1 } else { 0 };
            });
            self.packets = self
                .packets
                .drain()
                .sorted_by(|a, b| Ord::cmp(&a.0, &b.0))
                .skip(MAX_SAVED / 2)
                .collect();
        }
        if self.received.len() > MAX_RECEIVED {
            self.received = self
                .received
                .clone()
                .into_iter()
                .skip(MAX_RECEIVED / 2)
                .collect();
        }
    }

    fn set_last_valid(&mut self, packets: &[T]) {
        packets
            .iter()
            .map(|p| p.get())
            .max()
            .map(|max| self.last_id = max);
    }

    pub fn get_lost(&self) -> Vec<u32> {
        self.packets
            .keys()
            .max()
            .map(|max| (self.last_id + 1, *max))
            .map(|(min, max)| min..=max)
            .map(|range| range.filter(|i| !self.packets.contains_key(i)))
            .map(|filter| {
                let ids: Vec<u32> = filter.collect();
                ids
            })
            .unwrap_or_else(Vec::new)
    }

    fn get_valid(&mut self) -> Vec<T> {
        let mut i = self.last_id + 1;
        let mut vec: Vec<T> = Vec::new();
        while self.packets.contains_key(&i) {
            vec.push(self.packets.remove(&i).unwrap());
            i += 1;
        }
        vec
    }
}
