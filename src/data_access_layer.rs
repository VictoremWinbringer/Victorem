use std::net::{UdpSocket, SocketAddr};
use std::error::Error;
use crate::entities::{StatePacket, CommandPacket};
use bincode::{serialize, deserialize};

struct Address {
    address: SocketAddr
}

struct ClientSocket {
    socket: UdpSocket,
}

struct ServerSocket {
    socket: UdpSocket,
}

const TIMEOUT_IN_MILLIS: u64 = 1_000;
const MAX_DATAGRAM_SIZE: usize = 65_000;

impl ClientSocket {
    fn new(port: &str, server_address: &str) -> Result<ClientSocket, Box<dyn Error>> {
        use std::time::Duration;
        let local_address = format!("127.0.0.1:{}", port.trim());
        let remote_address = server_address.trim();
        let socket = UdpSocket::bind(&local_address)?;
        socket.connect(remote_address)?;
        socket.set_read_timeout(Some(Duration::from_millis(TIMEOUT_IN_MILLIS)))?;
        Ok(ClientSocket { socket })
    }

    fn read(&self, buffer: &mut [u8]) -> Result<usize, Box<dyn Error>> {
        let r = self.socket.recv(buffer)?;
        Ok(r)
    }

    fn write(&self, buf: &[u8]) -> Result<usize, Box<dyn Error>> {
        let r = self.socket.send(buf)?;
        Ok(r)
    }
}

impl ServerSocket {
    fn new(port: &str) -> Result<ServerSocket, Box<dyn Error>> {
        let local_address = format!("127.0.0.1:{}", port.trim());
        let socket = UdpSocket::bind(&local_address.trim())?;
        socket.set_read_timeout(Some(std::time::Duration::from_millis(TIMEOUT_IN_MILLIS)))?;
        Ok(ServerSocket { socket })
    }

    fn read(&self, buffer: &mut [u8]) -> Result<(usize, Address), Box<dyn Error>> {
        let (c, a) = self.socket.recv_from(buffer)?;
        Ok((c, Address { address: a }))
    }

    fn write(&self, buf: &[u8], addr: &Address) -> Result<usize, Box<dyn Error>> {
        let r = self.socket.send_to(buf, addr.address)?;
        Ok(r)
    }
}

struct BufferedServerSocket {
    socket: ServerSocket,
    buffer: [u8; MAX_DATAGRAM_SIZE],
}

impl BufferedServerSocket {
    fn new(port: &str) -> Result<BufferedServerSocket, Box<dyn Error>> {
        let socket = ServerSocket::new(port)?;
        let buffer = [0u8; MAX_DATAGRAM_SIZE];
        Ok(BufferedServerSocket { socket, buffer })
    }

    fn read(&mut self) -> Result<(Vec<u8>, Address), Box<dyn Error>> {
        let (c, a) = self.socket.read(&mut self.buffer)?;
        Ok((self.buffer[..c].into(), a))
    }

    fn write(&self, addr: &Address, buffer: &[u8]) -> Result<usize, Box<dyn Error>> {
        self.socket.write(buffer, addr)
    }
}

struct BufferedClientSocket {
    socket: ClientSocket,
    buffer: [u8; MAX_DATAGRAM_SIZE],
}

impl BufferedClientSocket {
    fn new(port: &str, server_address: &str) -> Result<BufferedClientSocket, Box<dyn Error>> {
        let socket = ClientSocket::new(port, server_address)?;
        let buffer = [0u8; MAX_DATAGRAM_SIZE];
        Ok(BufferedClientSocket { socket, buffer })
    }

    fn read(&mut self) -> Result<Vec<u8>, Box<dyn Error>> {
        let r = self.socket.read(&mut self.buffer)?;
        Ok(self.buffer[..r].into())
    }

    fn write(&self, buffer: &[u8]) -> Result<usize, Box<dyn Error>> {
        self.socket.write(buffer)
    }
}

pub struct TypedServerSocket {
    socket: BufferedServerSocket
}

impl TypedServerSocket {
    pub fn new(port: &str) -> Result<TypedServerSocket, Box<dyn Error>> {
        let socket = BufferedServerSocket::new(port)?;
        Ok(TypedServerSocket { socket })
    }

    pub fn read(&mut self) -> Result<(CommandPacket, Address), Box<dyn Error>> {
        let (b, a) = self.socket.read()?;
        let commands = deserialize(&b)?;
        Ok((commands, a))
    }

    pub fn write(&self, addr: &Address, state: &StatePacket) -> Result<usize, Box<dyn Error>> {
        let bytes = serialize(state)?;
        self.socket.write(addr, &bytes)
    }
}

pub struct TypedClientSocket {
    socket: BufferedClientSocket
}

impl TypedClientSocket {
    pub fn new(port: &str, server_address: &str) -> Result<TypedClientSocket, Box<dyn Error>> {
        let socket = BufferedClientSocket::new(port, server_address)?;
        Ok(TypedClientSocket { socket })
    }

    pub fn read(&mut self) -> Result<StatePacket, Box<dyn Error>> {
        let r = self.socket.read()?;
        let state = deserialize(&r)?;
        Ok(state)
    }

    pub fn write(&self, commands: &CommandPacket) -> Result<usize, Box<dyn Error>> {
        let bytes = serialize(commands)?;
        self.socket.write(&bytes)
    }
}

pub mod logger {
    use simplelog::*;
    use std::{fs::OpenOptions, io::Write};
    use std::error::Error;

    pub fn init() -> Result<(), Box<dyn Error>> {
        let mut file = OpenOptions::new().append(true).create(true).open("victorem_framework_logs.log")?;
        let write_logger = WriteLogger::new(LevelFilter::Info, Config::default(), file);
        CombinedLogger::init(
            vec![
                write_logger,
            ]
        )?;
        Ok(())
    }
}