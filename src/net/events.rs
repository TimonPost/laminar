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
    ConnectionError(ConnectionManagerError),
    Timeout,
    TooManyPacketsInFlight,
    TooManyPacketErrors,
    GracefullyDisconnected
}

#[derive(Debug, PartialEq, Clone)]
pub enum ConnectionClosedBy {
    LocalHost,
    RemoteHost
}

#[derive(Debug, PartialEq)]
pub enum DisconnectReason {
    ClosedBy(ConnectionClosedBy),
    UnrecoverableError(DestroyReason)
}

#[derive(Debug)]
pub struct ConnectionEvent<Event: std::fmt::Debug> {
    pub addr: SocketAddr,
    pub event: Event
}

#[derive(Debug)]
pub enum SendEvent {
    Connect(Box<[u8]>),
    Packet(Packet),
    Disconnect,
}

#[derive(Debug)]
pub enum ReceiveEvent {        
    Created,
    Connected(Box<[u8]>),
    Packet(Packet),
    Disconnected(DisconnectReason),
    Destroyed(DestroyReason),
}
