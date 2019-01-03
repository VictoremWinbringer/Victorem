use crate::entities::{CommandPacket, Exception, StatePacket};
use std::time::{Duration, SystemTimeError, SystemTime, UNIX_EPOCH};

pub trait IWithKey {
    fn get(&self) -> Duration;
    fn set(&mut self, key: Duration);
}

impl IWithKey for StatePacket {
    fn get(&self) -> Duration {
        self.session_key
    }

    fn set(&mut self, key: Duration) {
        self.session_key = key
    }
}

impl IWithKey for CommandPacket {
    fn get(&self) -> Duration {
        self.session_key
    }

    fn set(&mut self, key: Duration) {
        self.session_key = key
    }
}

pub fn new_key() -> Duration {
    match SystemTime::now()
        .duration_since(UNIX_EPOCH) {
        Ok(d) => d,
        Err(e) => e.duration(),
    }
}

pub struct Generator {
    key: Duration
}

impl Generator {
    pub fn new() -> Generator {
        Generator { key: new_key() }
    }

    pub fn generate(&self) -> Duration { self.key }
}

pub struct Filter {
    key: Duration
}

impl Filter {
    pub fn new(key: Duration) -> Filter {
        Filter { key }
    }

    pub fn is_valid(&self, value: &IWithKey) -> bool {
        self.key == value.get()
    }
}
