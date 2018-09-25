use std::io;
use std::net::{self, ToSocketAddrs, SocketAddr};
use std::collections::HashMap;

use bincode::{deserialize, serialize};
use super::{Packet, RawPacket, SocketState};

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
        let (len, _addr) = self.socket.recv_from(&mut self.recv_buffer)?;

        if len > 0 {
            // TODO: Remove unwrap and funnel result error types
            let raw_packet: RawPacket = deserialize(&self.recv_buffer[..len]).unwrap();
            let packet = self.state.process_received(_addr, &raw_packet);
            Ok(Some(packet))
        } else {
            Ok(None)
        }
    }

    pub fn send(&mut self, mut packet: Packet) -> io::Result<usize> {
        let (addr, payload) = self.state.pre_process_packet(packet);
        self.socket.send_to(&payload, addr)
    }

    pub fn set_nonblocking(&mut self, nonblocking: bool) -> io::Result<()> {
        self.socket.set_nonblocking(nonblocking)
    }
}
