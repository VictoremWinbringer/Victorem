use serde_derive::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct CommandPacket {
    id:u64,
    command: Vec<u8>,
}

impl CommandPacket {
   pub fn new(id:u64, command:Vec<u8>) -> CommandPacket {
        CommandPacket{id,command}
    }
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct StatePacket{
    id:u64,
    lost_ids: Vec<u64>,
    state: Vec<u8>,
}