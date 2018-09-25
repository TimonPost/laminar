use bincode::{deserialize, serialize};
use packet::Packet;
use std::io;
use std::net;
use std::net::ToSocketAddrs;

const BUFFER_SIZE: usize = 1024;

pub struct UdpSocket {
    socket: net::UdpSocket,
    state: SocketState,
    recv_buffer: [u8; BUFFER_SIZE],
}

impl UdpSocket {
    pub fn bind<A: ToSocketAddrs>(addr: A) -> io::Result<Self> {
        let socket = net::UdpSocket::bind(addr)?;
        let state = SocketState::new();

        Ok(UdpSocket {
            socket,
            state,
            recv_buffer: [0; BUFFER_SIZE],
        })
    }

    pub fn recv(&mut self) -> io::Result<Option<Packet>> {
        // TODO: Pass addr back with packet
        let (len, _addr) = self.socket.recv_from(&mut self.recv_buffer)?;

        if len > 0 {
            // TODO: Remove unwrap and funnel result error typesq
            let packet: Packet = deserialize(&self.recv_buffer).unwrap();
            Ok(Some(packet))
        } else {
            Ok(None)
        }
    }

    pub fn send(&mut self, packet: &Packet) -> io::Result<usize> {
        let addr = packet.addr();
        let buf = serialize(packet).unwrap();

        self.socket.send_to(&buf, addr)
    }

    pub fn set_nonblocking(&mut self, nonblocking: bool) -> io::Result<()> {
        self.socket.set_nonblocking(nonblocking)
    }
}

pub struct SocketState;

impl SocketState {
    pub fn new() -> Self {
        SocketState
    }
}
