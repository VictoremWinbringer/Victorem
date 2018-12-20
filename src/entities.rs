enum Command {
    Exit
}

struct CommandsPacket{
    id:u64,
    commands: Vec<Command>,
}

struct StatePacket{
    id:u64,
    lost_ids: Vec<u64>,
    state: Vec<u8>,
}