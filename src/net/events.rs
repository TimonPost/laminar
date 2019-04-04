use crate::packet::Packet;
use std::net::SocketAddr;

/// Events that can occur in `laminar` and that will be pushed through the `event_receiver` returned by `Socket::bind`.
#[derive(Debug, PartialEq)]
pub enum SocketEvent {
    /// A packet was received from a client.
    Packet(Packet),
    /// A new client connected.
    /// Clients are uniquely identified by the ip:port combination at this layer.
    Connect(SocketAddr),
    /// The client has been idling for a configurable amount of time.
    /// You can control the timeout in the config.
    Timeout(SocketAddr),
}
