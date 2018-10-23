mod tcp_channel;
mod udp_channel;

pub use self::tcp_channel::TcpChannel;
pub use self::udp_channel::UdpChannel;

use std::net::SocketAddr;
use std::io::Error;
use net_events::NetEvent;
use packet::Packet;


pub trait Channel {
    fn connect(&self, addr: &SocketAddr);
    fn local_addr(&self) -> Result<SocketAddr, Error>;
    fn start_receiving(&mut self) -> Result<(), Error>;
    fn send_to(&mut self, packet: Packet) -> Result<usize, Error>;
    fn events(&self) -> Vec<NetEvent>;
}

pub enum ChannelType {
    TCP,
    UDP,
}