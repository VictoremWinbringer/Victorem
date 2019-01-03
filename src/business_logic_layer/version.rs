use crate::entities::{CommandPacket, Exception, StatePacket};

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

pub struct VersionChecker;

impl VersionChecker {
    pub fn check(&self, data: &impl IWithVersion) -> Result<(), Exception> {
        if data.get() == PROTOCOL_VERSION {
            Ok(())
        } else {
            Err(Exception::BadProtocolVersion)
        }
    }

    pub fn get(&self) -> u8 {
        PROTOCOL_VERSION
    }
}
