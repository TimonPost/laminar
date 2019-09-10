use crate::packet::Packet;
use crate::error::ErrorKind;
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

#[derive(Debug)]
pub enum DisconnectReason {
    // TODO here can be all sorts of reasons
    Timeout,
    TooManyPacketsInFlight,
    TooManyPacketErrors,
    ClosedByClient,
    ClosedByHost,
}

#[derive(Debug)]
pub enum DestroyReason {
    // because disconnected
    Disconnected(DisconnectReason),
    // because wasnt able to connect
    HandshakeFailed(ErrorKind)
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