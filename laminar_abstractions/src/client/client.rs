use net_events::NetEvent;
use laminar::packet ::{Packet};
use laminar::infrastructure::DeliveryMethod;
use laminar::net::SocketAddr;
use super::{Channel, UdpChannel, TcpChannel, ChannelType};
pub struct Client {
    udp_channel: UdpChannel,
    tcp_channel: TcpChannel
}

impl Client {
    pub fn new() -> Client {
        unimplemented!()
    }

    pub fn close_connection()
    {
        // 1. Check if tcp connection is established an close the connection.
        // 2. Check if udp connection is established and close the connection.
        unimplemented!();
    }

    pub fn send_udp(&mut self, packet: &[u8], addr: SocketAddr,  delivery_method: DeliveryMethod) {
        // 1. Check if connection is already established.
        // 2. Congestion avoidance check.
        // 3. Send data over udp channel.
    }

    pub fn send_tcp(&mut self, payload: &[u8], addr: SocketAddr) {
        // 1. Check if connection is already established.
        // 2. Send data over tcp.
    }

    // Get all events that has happened, maybe build some iterator over events
    pub fn events(&self) -> Vec<NetEvent>{
        // 1. Get tcp events.
        // 2. Get udp events.
        // 3. Return the events.
        unimplemented!()
    }

    /// Start receiving.
    fn start_receiving() {
        // 1. start thread to listen for TCP data.
        // 2. start thread to listen for UDP data.
    }
}
