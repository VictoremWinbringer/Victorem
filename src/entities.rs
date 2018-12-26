use serde_derive::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct CommandPacket {
    pub  id: u64,
    pub  command: Vec<u8>,
}

impl CommandPacket {
    pub fn new(id: u64, command: Vec<u8>) -> CommandPacket {
        CommandPacket { id, command }
    }
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct StatePacket {
    pub  id: u64,
    pub lost_ids: Vec<u64>,
    pub state: Vec<u8>,
}