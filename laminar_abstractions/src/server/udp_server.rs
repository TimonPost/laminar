//! All UDP reflated logic for getting and sending data out to the other side.

use std::sync::mpsc::{self, Receiver, Sender};
use std::thread;
use std::io::{Error, Result, ErrorKind};
use net_events::NetEvent;
use packet::Packet;
use laminar::net::{UdpSocket, NetworkConfig, SocketAddr};
use super::{ProtocolServer, ServerConfig};

type MessageSender = Option<Sender<String>>;
type MessageReceiver = Option<Receiver<String>>;

pub struct UdpServer {
    rx: Receiver<NetEvent>,
    tx: Sender<NetEvent>,
    socket: UdpSocket,
}

impl UdpServer {
    pub fn new(config: ServerConfig, server_sender: Sender<NetEvent>) -> UdpServer
    {
        let socket = UdpSocket::bind(&config.udp_addr, NetworkConfig::default()).unwrap();
        socket.set_nonblocking(true);

        let (tx, rx) = mpsc::channel();

        UdpServer { tx: server_sender, rx, socket }
    }
}

impl ProtocolServer for UdpServer
{
    fn start_receiving(&mut self) {
        // 1. Setup listing udp thread.
        // 2. Try receive data from it.
        // 3. Push the event on to the tx channel.
        loop {
            // try receiving
            let result = self.socket.recv();

            let net_event = match result {
                Ok(Some(packet)) => {
                    self.tx.send(NetEvent::Packet(Packet::new(packet.addr(), packet.payload().to_vec())));
                }
                Ok(None) => {
                    NetEvent::Empty
                }
                Err(e) => {
                    NetEvent::Error(*e.kind())
                }
            };

            self.socket.events().iter().map(|e| self.tx.send(NetEvent::ClientEvent(event)));

            self.send_all();
        }
    }

    fn send_all(&mut self) {
        while let Ok(packet) = self.rx.try_recv() {
            match packet {
                NetEvent::Packet(packet) => { self.find_client_by_addr() },
                NetEvent::BroadCast { .. } => {},
                _ => { }
            }
        }
        // 1. check rx buffer
        // 2. send data to x clients depending on the event type.
        unimplemented!()
    }

    fn find_client_by_addr(&self, addr: &SocketAddr) -> Option<&()> {
        unimplemented!()
    }

    fn find_client_by_id<'a>(&self, client_id: u64) -> Option<&'a mut SocketAddr> {
        unimplemented!()
    }
}
