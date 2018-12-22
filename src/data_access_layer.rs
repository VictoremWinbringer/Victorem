use std::net::{UdpSocket, SocketAddr};

struct Address{
    address: SocketAddr
}
struct ClientSocket {
    socket: UdpSocket
}

struct ServerSocket {
    socket: UdpSocket
}

const TIMEOUT_IN_MILLIS: u64 = 1000;

impl ClientSocket {
    fn new(port: &str, server_address: &str) -> Result<ClientSocket, Box<dyn std::error::Error>> {
        use std::time::Duration;
        let local_address = format!("127.0.0.1:{}", port.trim());
        let remote_address = server_address.trim();
        let socket = UdpSocket::bind(&local_address)?;
        socket.connect(remote_address)?;
        socket.set_read_timeout(Some(Duration::from_millis(TIMEOUT_IN_MILLIS)))?;
        Ok(ClientSocket { socket })
    }

    fn read(&self, buffer:&mut [u8]) -> Result<usize,Box<dyn std::error::Error>>{
      let r =  self.socket.recv(buffer)?;
        Ok(r)
    }

    fn write(&self, buf: &[u8]) -> Result<usize,Box<dyn std::error::Error>>{
      let r =  self.socket.send(buf)?;
        Ok(r)
    }
}

impl ServerSocket {
    fn new(port: &str) -> Result<UdpSocket, Box<dyn std::error::Error>> {
        let local_address = format!("127.0.0.1:{}", port.trim());
        let socket = UdpSocket::bind(&local_address.trim())?;
        socket.set_read_timeout(Some(std::time::Duration::from_millis(TIMEOUT_IN_MILLIS)))?;
        Ok(socket)
    }

    fn read(&self, buffer:&mut [u8]) -> Result<(usize,Address),Box<dyn std::error::Error>>{
        let (c, a) =  self.socket.recv_from(buffer)?;
        Ok((c, Address{address: a}))
    }

    fn write(&self, buf: &[u8], addr: Address) -> Result<usize,Box<dyn std::error::Error>>{
        let r =  self.socket.send_to(buf,addr.address)?;
        Ok(r)
    }
}

mod parser {
    use bincode::{serialize, deserialize};

    fn serialize_commands(commands: &crate::entities::CommandsPacket) -> Vec<u8> {
        serialize(commands).map_err(|e| crate::data_access_layer::logger::error(&format!("serialize_commands error {}", e)))
            .unwrap_or(Vec::new())
    }
}

pub mod logger {
    use log::{warn, error};
    use simplelog::*;
    use std::{fs::OpenOptions, io::Write};


    pub fn init() -> Result<(), Box<dyn std::error::Error>> {
        let mut file = OpenOptions::new().append(true).create(true).open("victorem_framework_logs.log")?;
        let write_logger = WriteLogger::new(LevelFilter::Info, Config::default(), file);
      CombinedLogger::init(
            vec![
                write_logger,
            ]
        )?;
        Ok(())
    }

    pub fn error(error: &str) {
        error!("{}", error)
    }

    pub fn warn(warn: &str) {
        warn!("{}", warn)
    }
}
