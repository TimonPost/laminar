use crate::net::managers::ConnectionManagerError;
use crate::packet::Packet;
use std::net::SocketAddr;

/// Events that can occur in `laminar` for a active connection.
#[derive(Debug)]
pub enum ReceiveEvent {
    /// When the connection is actually created and added to the active connections list.
    /// Next possible event for connection is: `Connected` or `Destroyed`.
    Created,
    /// When `ConnectionManager` successfully establishes connection.
    /// Next possible event is: `Packet` or `Disconnected`.
    Connected(Box<[u8]>),
    /// When connection is in Connected state, it can actually start receiving packets.
    /// Next possible event is: `Packet` or `Disconnected`.
    Packet(Packet),
    /// When connection, that was previously in a connected state, is disconnected
    /// it can either be disconnected by `ConnectionManager` in this case it is a 'clean' disconnect, where the initiator of disconnect is also specified
    /// or it can be closed by `SocketManager` if it decides to do so
    Disconnected(DisconnectReason),
    /// When it is removed from the active connections list.
    /// Connection can be destroyed when the disconnect is initiated by `ConnectionManager`, or `SocketManager` decided to destroy it.
    Destroyed(DestroyReason),
}

/// Events that are received from the user.
#[derive(Debug)]
pub enum SendEvent {
    /// Initiate connect request, this will call `ConnectionManager.connect` method.
    Connect(Box<[u8]>),
    /// Send packet to the remote host.
    Packet(Packet),
    /// Initiate disconnect, this will call `ConnectionManager.disconnect` method.
    Disconnect,
}

/// Provides a reason why the connection was destroyed.
#[derive(Debug, PartialEq, Clone)]
pub enum DestroyReason {
    /// When `SocketManager` decided to destroy a connection for error that arrived from `ConnectionManager`.
    ConnectionError(ConnectionManagerError),
    /// After `Config.idle_connection_timeout` connection had no activity.
    Timeout,
    /// If there are too many non-acked packets in flight `Config.max_packets_in_flight`.
    TooManyPacketsInFlight,
    /// When `ConnectionManager` changed to `Disconnected` state.
    GracefullyDisconnected,
}

/// Provides convenient enum, to specify either Local or Remote host
#[derive(Debug, PartialEq, Clone)]
pub enum TargetHost {
    /// Represents the localhost
    LocalHost,
    /// Represents the remote host
    RemoteHost,
}

/// Disconnect reason, received by connection
#[derive(Debug, PartialEq)]
pub enum DisconnectReason {
    /// Disconnect was initiated by the local or remote host
    ClosedBy(TargetHost),
    /// Socket manager decided to destroy connection for provided reason
    Destroying(DestroyReason),
}

/// Relate send or receive events together with address.
#[derive(Debug)]
pub struct ConnectionEvent<Event: std::fmt::Debug>(pub SocketAddr, pub Event);
