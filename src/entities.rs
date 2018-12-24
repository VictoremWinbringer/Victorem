use serde_derive::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct CommandsPacket{
    id:u64,
    commands: Vec<Vec<u8>>,
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct StatePacket{
    id:u64,
    lost_ids: Vec<u64>,
    state: Vec<u8>,
}