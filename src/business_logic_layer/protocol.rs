use crate::entities::{CommandPacket, Exception, StatePacket};

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

pub struct ProtocolChecker;

impl ProtocolChecker {
    pub fn check(&self, data: &impl IWithProtocol) -> Result<(), Exception> {
        if data.get() == PROTOCOL_ID {
            Ok(())
        } else {
            Err(Exception::NotValidIdError)
        }
    }

    pub fn get(&self) -> u8 {
        PROTOCOL_ID
    }
}
