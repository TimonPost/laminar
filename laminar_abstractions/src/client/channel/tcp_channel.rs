//! All TCP reflated logic for getting and sending data out to the other side.

use std::net::{SocketAddr, TcpStream, TcpListener};
use std::sync::mpsc::{self, Receiver, Sender};
use std::io::Error;
use net_events::NetEvent;
use super::Channel;
use packet::Packet;

pub struct TcpChannel {
    socket: TcpListener,
    stream: TcpStream,
    rx: Receiver<NetEvent>,
    tx: Sender<NetEvent>,
}

impl TcpChannel {
    pub fn new() -> TcpChannel
    {
        unimplemented!()
    }
}

impl Channel for TcpChannel
{
    fn connect(&self, addr: &SocketAddr) {
        // connect to the other side with tcp
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
