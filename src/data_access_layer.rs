use crate::entities::{CommandPacket, Exception, StatePacket};
use bincode::{deserialize, serialize};
use std::net::{SocketAddr, UdpSocket, ToSocketAddrs};

struct ClientSocket {
    socket: UdpSocket,
}

struct ServerSocket {
    socket: UdpSocket,
}

pub const MAX_DATAGRAM_SIZE: usize = 64_000;

impl ClientSocket {
    fn new(port: u16, server_address: impl ToSocketAddrs) -> Result<ClientSocket, Exception> {
        let local_address = format!("0.0.0.0:{}", port);
        let socket = UdpSocket::bind(&local_address)?;
        socket.connect(server_address)?;
        socket.set_nonblocking(true)?;
        Ok(ClientSocket { socket })
    }

    fn read(&self, buffer: &mut [u8]) -> Result<usize, Exception> {
        let c = self.socket.recv(buffer)?;
        Ok(c)
    }

    fn write(&self, buf: &[u8]) -> Result<usize, Exception> {
        let r = self.socket.send(buf)?;
        Ok(r)
    }
}

impl ServerSocket {
    fn new(port: u16) -> Result<ServerSocket, Exception> {
        let local_address = format!("0.0.0.0:{}", port);
        let socket = UdpSocket::bind(&local_address.trim())?;
        socket.set_nonblocking(true)?;
        Ok(ServerSocket { socket })
    }

    fn read(&self, buffer: &mut [u8]) -> Result<(usize, SocketAddr), Exception> {
        let (c, a) = self.socket.recv_from(buffer)?;
        Ok((c, a))
    }

    fn write(&self, buf: &[u8], addr: &SocketAddr) -> Result<usize, Exception> {
        let r = self.socket.send_to(buf, addr)?;
        Ok(r)
    }
}

struct BufferedServerSocket {
    socket: ServerSocket,
    buffer: Vec<u8>,
}

impl BufferedServerSocket {
    fn new(port: u16) -> Result<BufferedServerSocket, Exception> {
        let socket = ServerSocket::new(port)?;
        let buffer = vec![0u8; MAX_DATAGRAM_SIZE];
        Ok(BufferedServerSocket { socket, buffer })
    }

    fn read(&mut self) -> Result<(Vec<u8>, SocketAddr), Exception> {
        let (c, a) = self.socket.read(&mut self.buffer)?;
        Ok((self.buffer[..c].into(), a))
    }

    fn write(&self, addr: &SocketAddr, buffer: &[u8]) -> Result<usize, Exception> {
        self.socket.write(buffer, addr)
    }
}

struct BufferedClientSocket {
    socket: ClientSocket,
    buffer: Vec<u8>,
}

impl BufferedClientSocket {
    fn new(port: u16, server_address: impl ToSocketAddrs) -> Result<BufferedClientSocket, Exception> {
        let socket = ClientSocket::new(port, server_address)?;
        let buffer = vec![0u8; MAX_DATAGRAM_SIZE];
        Ok(BufferedClientSocket { socket, buffer })
    }

    fn read(&mut self) -> Result<Vec<u8>, Exception> {
        let r = self.socket.read(&mut self.buffer)?;
        Ok(self.buffer[..r].into())
    }

    fn write(&self, buffer: &[u8]) -> Result<usize, Exception> {
        self.socket.write(buffer)
    }
}

pub struct TypedServerSocket {
    socket: BufferedServerSocket,
}

impl TypedServerSocket {
    pub fn new(port: u16) -> Result<TypedServerSocket, Exception> {
        let socket = BufferedServerSocket::new(port)?;
        Ok(TypedServerSocket { socket })
    }

    pub fn read(&mut self) -> Result<(CommandPacket, SocketAddr), Exception> {
        let (b, a) = self.socket.read()?;
        let commands = deserialize(&b)?;
        Ok((commands, a))
    }

    pub fn write(&self, addr: &SocketAddr, state: &StatePacket) -> Result<usize, Exception> {
        let bytes = serialize(state)?;
        self.socket.write(addr, &bytes)
    }
}

pub struct TypedClientSocket {
    socket: BufferedClientSocket,
}

impl TypedClientSocket {
    pub fn new(port: u16, server_address: impl ToSocketAddrs) -> Result<TypedClientSocket, Exception> {
        let socket = BufferedClientSocket::new(port, server_address)?;
        Ok(TypedClientSocket { socket })
    }

    pub fn read(&mut self) -> Result<StatePacket, Exception> {
        let r = self.socket.read()?;
        let state = deserialize(&r)?;
        Ok(state)
    }

    pub fn write(&self, commands: &CommandPacket) -> Result<usize, Exception> {
        let bytes = serialize(commands)?;
        self.socket.write(&bytes)
    }
}

pub struct Cache {
    data: Vec<CommandPacket>,
}

impl Cache {
    const MAX_SAVED: usize = 200;
    pub fn new() -> Cache {
        Cache { data: Vec::new() }
    }
    pub fn add(&mut self, command: CommandPacket) {
        if self.data.len() > Cache::MAX_SAVED {
            self.data = self
                .data
                .clone()
                .into_iter()
                .skip(Cache::MAX_SAVED / 2)
                .collect();
        }
        self.data.push(command);
    }

    pub fn get_max_id(&self) -> u32 {
        self.data.iter()
            .map(|x| x.id)
            .max()
            .unwrap_or(0)
    }

    pub fn get(&mut self, id: u32) -> Option<CommandPacket> {
        self.data
            .iter()
            .position(|c| c.id == id)
            .map(|i| self.data[i].clone())
    }

    pub fn get_range(&mut self, ids: &[u32]) -> Vec<CommandPacket> {
        let mut vec = Vec::<CommandPacket>::new();
        for id in ids {
            self.get(*id).map(|p| vec.push(p));
        }
        vec
    }
}
