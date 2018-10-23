//! All UDP reflated logic for getting and sending data out to the other side.

use std::net::{SocketAddr, TcpStream, TcpListener};
use std::io::{self, Error};
use std::time::Duration;
use std::sync::mpsc::{self, Receiver, Sender};
use laminar::net::{UdpSocket, NetworkConfig};
use packet::Packet;
use net_events::NetEvent;
use super::Channel;

pub struct UdpChannel {
    socket: UdpSocket,
    rx: Receiver<()>,
    tx: Sender<()>,
}

impl UdpChannel {
    pub fn new() -> Self {
        let config = NetworkConfig::default();
        let mut udp_socket: UdpSocket = UdpSocket::bind("127.0.0.1:12345", config).unwrap();
        udp_socket.set_nonblocking(false);

        let (tx,rx) = mpsc::channel();

        UdpChannel { socket: udp_socket, rx, tx }
    }
}

impl Channel for UdpChannel {
    fn connect(&self, addr: &SocketAddr) {
        // connect to the other side by the given socket addr with `bind()`.
        unimplemented!()
    }

    fn local_addr(&self) -> Result<SocketAddr, Error> {
        // get socket addr from udpsocket
        unimplemented!()
    }

    fn start_receiving(&mut self) -> Result<(), Error> {
        // 1. Try receiving packets.
        // 2. Push them onto the rx channel in the shape of an event.
        // 3. Continue receiving
        unimplemented!()
    }

    fn send_to(&mut self, packet: Packet) -> Result<usize, Error> {
        // 1. Sent packet directly to other side.
        unimplemented!()
    }

    fn events(&self) -> Vec<NetEvent> {
        // 1. read all events from `rx`
        // 2. return the events.
        unimplemented!()
    }
}