use crate::packet::Packet;
use crate::net::managers::ConnectionManagerError;
use std::net::SocketAddr;

/// Events that can occur in `laminar` and that will be pushed through the `event_receiver` returned by `Socket::bind`.
// #[derive(Debug, PartialEq)]
// pub enum SocketEvent {
//     /// A packet was received from a client.
//     Packet(Packet),
//     /// A new client connected.
//     /// Clients are uniquely identified by the ip:port combination at this layer.
//     Connect(SocketAddr),
//     /// The client has been idling for a configurable amount of time.
//     /// You can control the timeout in the config.
//     Timeout(SocketAddr),
// }

#[derive(Debug, PartialEq, Clone)]
pub enum DestroyReason {
    // because wasnt able to connect
    HandshakeFailed(ConnectionManagerError),
    Timeout,
    TooManyPacketsInFlight,
    TooManyPacketErrors,
}

#[derive(Debug, PartialEq)]
pub enum DisconnectReason {
    ClosedByClient,
    ClosedByHost,
    UnrecoverableError(DestroyReason)
}

#[derive(Debug, PartialEq)]
pub enum SocketEvent {    
    Created(SocketAddr),
    Connected,
    Packet(Packet),
    Disconnected(DisconnectReason),
    Destroyed(DestroyReason),
}

#[derive(Debug)]
pub enum ConnectionReceiveEvent {
    /// When the connection is actually added to active connections list.
    Created,
    /// When connection manager changes to connected state
    Connected, 
    /// Actual received packet, this should only occure after Connected state
    Packet(Packet),
    /// When connection manager changes to disconnected state
    Disconnected(DisconnectReason),
    /// When connection is actually removed from connections list.
    Destroyed(DestroyReason)
}

#[derive(Debug)]
pub enum ConnectionSendEvent {
    Connect,
    Packet(Packet),
    Disconnect,
}