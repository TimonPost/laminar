use laminar::error::NetworkResult;
use packet::Packet;
use laminar::net::{UdpSocket, NetworkConfig, SocketAddr, VirtualConnection};
use laminar::infrastructure::DeliveryMethod;
use net_events::NetEvent;
use super::{ServerConfig, TcpServer, UdpServer, ProtocolServer};
use std::thread::{self, JoinHandle};
use std::sync::mpsc::{self, Receiver, Sender};
use std::time::Duration;

// Server where clients can connect to.
pub struct Server {
    config: ServerConfig,
    udp_server: UdpServer,
    tcp_server: TcpServer
}

impl Server {
    pub fn new(config: ServerConfig) -> Self {
        let (tx, rx) = mpsc::channel();
        Server { config: config.clone(), tcp_server: TcpServer::new(config.clone(), tx, rx), udp_server: UdpServer::new(config.clone())}
    }

    pub fn run(&mut self) {

        if self.config.enable_tcp {
            TcpServer::start_accepting(self.tcp_server.tcp_clients.clone(), self.config.clone());
            self.tcp_server.start_receiving();
            let a: Option<i32> = Some(0);
            let b: Option<i32> = a.to_owned();
        }

        if self.config.enable_udp {
            self.udp_server.start_receiving();
        }
    }

    pub fn shutdown(&mut self) {
        // 1. shutdown all connections udp/tcp
    }

    pub fn broad_cast_tcp(&mut self, payload: &[u8], addr: SocketAddr) -> NetworkResult<()>
    {
        // 1. Loop tcp clients.
        // 2. Send data to all clients.
        unimplemented!()
    }

    pub fn broad_cast_upd(&mut self, payload: &[u8], addr: SocketAddr, delivery_method: DeliveryMethod) -> NetworkResult<()>
    {
        // 1. Loop udp clients.
        // 2. Send data to all clients.
        unimplemented!()
    }

    pub fn send_udp(&mut self, packet: Packet) -> NetworkResult<()>
    {
        unimplemented!()
    }

    pub fn send_tcp(&mut self, packet: Packet) -> NetworkResult<()>
    {
        unimplemented!()
    }

    fn find_client_by_addr<'a>(addr: &SocketAddr) -> Option<&'a mut VirtualConnection> {
        unimplemented!()
    }

    fn find_client_by_id<'a>(client_id: u64) -> Option<&'a mut VirtualConnection> {
        unimplemented!()
    }

    fn disconnect<'a>(client_id: u64) -> Option<&'a mut VirtualConnection> {
        unimplemented!()
    }
}