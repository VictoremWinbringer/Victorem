use std::net::{UdpSocket, SocketAddr};
use std::error::Error;
use crate::entities::{StatePacket, CommandPacket};
use bincode::{serialize, deserialize};
use std::fmt::Display;
use std::fmt::Formatter;

#[derive(Debug)]
struct NotProtocolPacketError;

impl Error for NotProtocolPacketError {}

impl Display for NotProtocolPacketError {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        write!(f, "Error - packet don't have correct protocol id")
    }
}

 struct ClientSocket {
    socket: UdpSocket,
}

struct ServerSocket {
    socket: UdpSocket,
}

const TIMEOUT_IN_MILLIS: u64 = 1_000;
const MAX_DATAGRAM_SIZE: usize = 65_000;
const PROTOCOL_ID: u8 = 8;
const PROTOCOL_ID_SIZE_IN_BYTES:usize = 1;

impl ClientSocket {
    fn new(port: &str, server_address: &str) -> Result<ClientSocket, Box<dyn Error>> {
        use std::time::Duration;
        let local_address = format!("127.0.0.1:{}", port.trim());
        let remote_address = server_address.trim();
        let socket = UdpSocket::bind(&local_address)?;
        socket.connect(remote_address)?;
        socket.set_nonblocking(true)?;
        Ok(ClientSocket { socket })
    }

    fn read(&self, buffer: &mut [u8]) -> Result<usize, Box<dyn Error>> {
        let c = self.socket.recv(buffer)?;
        if c >= PROTOCOL_ID_SIZE_IN_BYTES && buffer[0] == PROTOCOL_ID {
            Ok(c)
        } else {
            Err(Box::new(NotProtocolPacketError))
        }
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
        socket.set_nonblocking(true)?;
        Ok(ServerSocket { socket })
    }

    fn read(&self, buffer: &mut [u8]) -> Result<(usize, SocketAddr), Box<dyn Error>> {
        let (c, a) = self.socket.recv_from(buffer)?;
        if c >= PROTOCOL_ID_SIZE_IN_BYTES && buffer[0] == PROTOCOL_ID {
            Ok((c, a))
        } else {
            Err(Box::new(NotProtocolPacketError))
        }
    }

    fn write(&self, buf: &[u8], addr: &SocketAddr) -> Result<usize, Box<dyn Error>> {
        let r = self.socket.send_to(buf, addr)?;
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

    fn read(&mut self) -> Result<(Vec<u8>, SocketAddr), Box<dyn Error>> {
        let (c, a) = self.socket.read(&mut self.buffer)?;
        Ok((self.buffer[PROTOCOL_ID_SIZE_IN_BYTES..c].into(), a))
    }

    fn write(&self, addr: &SocketAddr, buffer: &[u8]) -> Result<usize, Box<dyn Error>> {
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
        Ok(self.buffer[PROTOCOL_ID_SIZE_IN_BYTES..r].into())
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

    pub fn read(&mut self) -> Result<(CommandPacket, SocketAddr), Box<dyn Error>> {
        let (b, a) = self.socket.read()?;
        let commands = deserialize(&b)?;
        Ok((commands, a))
    }

    pub fn write(&self, addr: &SocketAddr, state: &StatePacket) -> Result<usize, Box<dyn Error>> {
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

    pub fn init(log_level: LevelFilter) -> Result<(), Box<dyn Error>> {
        let mut file = OpenOptions::new().append(true).create(true).open("victorem_framework_logs.log")?;
        let write_logger = WriteLogger::new(log_level, Config::default(), file);
        CombinedLogger::init(
            vec![
                write_logger,
            ]
        )?;
        Ok(())
    }
}