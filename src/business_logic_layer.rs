use crate::entities::{CommandPacket, StatePacket, Exception};
//TODO: disconnect after 10 seconds and que to send one packet in 30 ms and send lost ids

mod versions {
    use crate::entities::{StatePacket, CommandPacket, Exception};

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

    fn check(data: &impl IWithVersion) -> Result<(), Exception> {
        if data.get() == PROTOCOL_VERSION {
            Ok(())
        } else {
            Err(Exception::BadProtocolVersion)
        }
    }

    fn set(data: &mut impl IWithVersion) {
        data.set(PROTOCOL_VERSION)
    }
}

mod protocol_id {
    use crate::entities::{StatePacket, CommandPacket, Exception};

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

    fn check(data: &impl IWithProtocol) -> Result<(), Exception> {
        if data.get() == PROTOCOL_ID {
            Ok(())
        } else {
            Err(Exception::BadProtocolVersion)
        }
    }

    pub fn set(data: &mut impl IWithProtocol) {
        data.set(PROTOCOL_ID)
    }
}

mod id {
    use crate::entities::{StatePacket, CommandPacket, Exception};

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
        id: u32
    }

    impl Generator {
        fn set(&mut self, data: &mut impl IWithId) {
            data.set(self.id);
            self.id += 1;
        }
    }

    pub struct Filter {
        id: u32
    }

    impl Filter {
        pub fn valid(&self, data: &impl IWithId) -> Result<(), Exception> {
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

        pub fn filter_and_set(&mut self, data: &impl IWithId) -> Result<(), Exception> {
            self.valid(data)?;
            self.set(data.get());
            Ok(())
        }
    }
}

mod arrange {
    use crate::business_logic_layer::id::{IWithId, Filter};
    use crate::entities::Exception;
    use std::collections::{HashMap, VecDeque};

    struct Arranger<T: IWithId> {
        filter: Filter,
        packets: HashMap<u32, T>,
    }

    impl<T: IWithId> Arranger<T> {
        pub fn add(&mut self, data: T) -> Result<(), Exception> {
            self.filter.valid(&data)?;
            self.packets.entry(data.get())
                .or_insert(data);
            Ok(())
        }

        pub fn get_valid_and_lost(&mut self) -> (Vec<T>, Vec<u32>) {
            let packets = self.find_valid();
            let lost_ids = self.find_lost();
            self.set_last_valid(&packets);
            (packets, lost_ids)
        }

        fn set_last_valid(&mut self, packets: &Vec<T>) {
            packets.iter()
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
                .map(|range|
                    range.filter(|i| !self.packets.contains_key(i)))
                .map(|filter| {
                    let ids: Vec<u32> = filter.collect();
                    ids
                }).unwrap_or(Vec::new())
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
}

mod time {
    use std::time::{Duration, Instant};
    use std::thread;

    struct Timer {
        time: Duration,
        instant: Instant,
    }

    impl Timer {
        fn wait(&mut self) {
            let elapsed = self.instant.elapsed();
            self.time.checked_sub(elapsed)
                .and_then(|d| if d == Duration::new(0, 0) {
                    None
                } else { Some(d) })
                .map(|d| thread::sleep(d));
            self.instant = Instant::now();
        }
    }
}

pub struct Client {
//    id: u32,
//    socket: TypedClientSocket,
//    last_recv_id: u32,
//    send_packets: HashMap<u32, CommandPacket>,
}

impl Client {
    fn send(&mut self, command: Vec<u8>) -> CommandPacket {unimplemented!()}
    fn recv(&mut self, state: StatePacket) -> Vec<u8> {unimplemented!()}
//    const MAX_SAVED_PACKETS: usize = 6000;
//
//    pub fn new(port: &str, server_address: &str) -> Result<Client, Exception> {
//        let socket = TypedClientSocket::new(port, server_address)?;
//        Ok(Client { id: 1, socket, last_recv_id: 0, send_packets: HashMap::new() })
//    }
//
//    fn write(&mut self, command: CommandPacket) -> Result<usize, Exception> {
//        self.socket.write(&command)
//    }
//
//    fn read(&mut self) -> Result<StatePacket, Exception> {
//        self.socket.read()
//    }
//
//    fn recv_ordered(&mut self) -> Result<StatePacket, Exception> {
//        let packet = self.read()?;
//        if packet.id <= self.id {
//            Err(Exception::NotOrderedPacketError)
//        } else {
//            self.id = packet.id;
//            Ok(packet)
//        }
//    }
//
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
//
//    //////////////////////////////////////////////////////////////////////////////////////////////////////////////
//    fn send_and_remember(&mut self, command: CommandPacket) -> Result<usize, Exception> {
//        if self.send_packets.len() > Client::MAX_SAVED_PACKETS {
//            self.send_packets = self.send_packets.clone()
//                .into_iter()
//                .skip(Client::MAX_SAVED_PACKETS / 2)
//                .collect();
//        }
//        self.send_packets.entry(command.id).or_insert(command.clone());
//        self.write(command)
//    }
//
//    fn send_with_id(&mut self, id: u32, command: Vec<u8>) -> Result<usize, Exception> {
//        self.send_and_remember(CommandPacket {
//            protocol_id: 0,
//            protocol_version: 0,
//            id: id,
//            command: command,
//        })
//    }
//
//    fn send_and_increase_last_send_id(&mut self, command: Vec<u8>) -> Result<usize, Exception> {
//        let id = self.id.clone();
//        self.id += 1;
//        self.send_with_id(id, command)
//    }
//
//    pub fn send(&mut self, command: Vec<u8>) -> Result<usize, Exception> {
//        self.send_and_increase_last_send_id(command)
//    }
}


struct Server {
    // socket: TypedServerSocket
}

impl Server {
    fn send(&mut self, state:Vec<u8>) -> StatePacket {unimplemented!()}
    fn recv(&mut self, command:CommandPacket) -> Vec<u8> {unimplemented!()}
}
