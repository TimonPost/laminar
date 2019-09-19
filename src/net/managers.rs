pub use crate::either::Either;
pub use crate::net::events::{DestroyReason, TargetHost};
pub use crate::packet::{
    DeliveryGuarantee, GenericPacket, OrderingGuarantee, OutgoingPacket, PacketType,
};
pub use crate::ErrorKind;
use std::fmt::Debug;
use std::net::SocketAddr;
use std::time::Instant;

/// At any given moment, any connection can be only in these states.
/// These states are only managed through `ConnectionManager`, and define behaviour for sending and receiving packets.
/// Only these state transition is allowed:
/// | Old          | New          |
/// | ----------   | ----------   |
/// | Connecting   | Connected    |
/// | Connecting   | Disconnected |
/// | Connected    | Disconnected |
/// If these rules are not satisfied, panic! will be called.
/// Each state specifies what can and cannot be done:
/// * Connecting - This is initial state when socket is created, at this moment no packets can be sent or received from user,
/// in this state only `ConnectionManager` is able to receive and sent packets to properly initiate connection.
/// * Connected - Only in this state all packets will be sent or received between peers.
/// * Disconnected - in this state `ConnectionManager` is not able to send or receive any packets. Connection will be destroyed immediatelly.
#[derive(Debug, PartialEq, Clone)]
pub enum ConnectionState {
    Connecting,
    Connected(Box<[u8]>),
    Disconnected(TargetHost),
}

impl ConnectionState {
    /// Tries to change current state and returns old state if successfully changed.
    pub fn try_change(&mut self, new: &Self) -> Option<Self> {
        match (&self, &new) {
            (ConnectionState::Connecting, ConnectionState::Connected(_))
            | (ConnectionState::Connecting, ConnectionState::Disconnected(_))
            | (ConnectionState::Connected(_), ConnectionState::Disconnected(_)) => {
                Some(std::mem::replace(self, new.clone()))
            }
            _ => None,
        }
    }
}

impl Default for ConnectionState {
    fn default() -> Self {
        Self::Connecting
    }
}

/// Generic error type, that is used by ConnectionManager implementation.
#[derive(Debug, PartialEq, Clone)]
pub enum ConnectionManagerError {
    /// Something really bad has happened, this is not a recoverable error, and the connection should be destroyed.
    Fatal(String),
    /// Something unexpected has happened, but the connection is still in a valid state.
    /// `SocketManager` can decide when to destroy the connection if two many warnings are propagated from the same connection in a short amount of time.
    Warning(String), // TODO: is it enought? or maybe we need more fields?
}

/// It abstracts pure UDP packets, and allows to implement Connected/Disconnected states.
/// This table summary shows where exactly ConnectionManager sits in between different layers.
/// | Abstraction layer | Capabilities                                                                                                                                                         |
/// |-------------------|----------------------------------------------------------------------------------------------------------------------------------------------------------------------|
/// | Application       | Can receive these events: Created->Connected(data)->Packet(data)->Disconnected(reason)->Destroyed(reason). Can send these events: Connect(data)->Packet->Disconnect. |
/// | ConnectionManager | Receives all, except user packets, and can report state updates, and generate new packets via `update` method                                                        |
/// | Laminar           | Adds/Removes headers to packets, so that it could provides reliability, ordering, fragmentation, etc.. capabilities.                                                 |
/// | ConnectionManager | May change raw incoming and outgoing bytes to apply encryption, compression, etc.                                                                                    |
///
/// It tries to maintain a valid connection state, and it can't decide when to destroy itself, only when it changes to disconnected, it will be destroyed later.
/// From the point of view of connection manager, laminar's header + payload is interpreted as user data.
/// Distinction between user packet and the protocol-specific packet is encoded in laminar's packet header.
/// Preprocess/Postprocess and Update methods always accept temporary buffer of size `Config.receive_buffer_max_size` that can be used as output.
pub trait ConnectionManager: Debug + Send {
    /// When the instance of the connection manager is created, the `update` method will be called, before any other method.
    /// This function should be called frequently, even if there are no packets to send or receive.
    /// It will always be called last, after all, other methods are called, in the main laminar`s loop.
    /// It can generate all kinds of packets: heartbeat, user or connection protocol packets.
    /// (maybe heartbeat functionality should be moved here?)
    /// It will be called in the loop as long, as it returns any results. E.g. `connect` method may generate multiple results: change state and send packet.
    fn update<'a>(
        &mut self,
        buffer: &'a mut [u8],
        time: Instant,
    ) -> Option<Result<Either<GenericPacket<'a>, ConnectionState>, ConnectionManagerError>>;

    /// This will be called for all incoming data, including packets that were resent by remote host.
    /// If the packet is accepted by laminar's reliability layer `process_protocol_data` will be called immediately.
    /// It should return a slice where header + payload exists
    fn preprocess_incoming<'a, 'b>(
        &mut self,
        data: &'a [u8],
        buffer: &'b mut [u8],
    ) -> Result<&'b [u8], ConnectionManagerError>
    where
        'a: 'b;

    /// This will be called for all outgoing data, including packets that are resent.
    /// Dropped packets will also go through here.
    /// Accepts full packet: header + payload
    fn postprocess_outgoing<'a, 'b>(&mut self, data: &'a [u8], buffer: &'b mut [u8]) -> &'b [u8]
    where
        'a: 'b;

    /// This will be called only for incoming protocol-specific packets after laminar's reliability layer accepted it.
    /// This is a convenient place to process actual logic because it is filtered by laminar's reliability layer and it accepts only `PacketType::Connection` messages.
    fn process_protocol_data(&mut self, data: &[u8]) -> Result<(), ConnectionManagerError>;

    /// This will be invoked when a user sends connect request,
    /// Some protocols might provide a way to pass initial connection data, hence the `data` field.
    /// This method can only be called when the connection is in `Connecting` state
    fn connect(&mut self, data: Box<[u8]>);

    /// This will be invoked when a user sends SendEvent::Disconnect request.
    fn disconnect(&mut self);
}

/// Tracks all sorts of global statistics and can decided whether to create a `ConnectionManager` for new connections or not.
/// Also decides when connections should be destroyed even if they are in a connected state.
pub trait SocketManager: Debug + Send {
    /// Decide if it is possible to accept/create new remote connection, this is invoked when a message from unknown address arrives.
    fn accept_remote_connection(
        &mut self,
        addr: &SocketAddr,
        raw_bytes: &[u8],
    ) -> Option<Box<dyn ConnectionManager>>;
    /// Decide if it is possible to accept/create new local connection, this is invoked when any user event is received for unknown address.
    fn accept_local_connection(&mut self, addr: &SocketAddr) -> Option<Box<dyn ConnectionManager>>;

    /// Returns a list of connections that the socket manager decided to destroy, along with a destroying reason
    fn collect_connections_to_destroy(&mut self) -> Option<Vec<(SocketAddr, DestroyReason)>>;

    /// All sorts of statistics might be useful here to help to decide whether a new connection can be created or not
    fn track_connection_error(&mut self, addr: &SocketAddr, error: &ErrorKind, error_context: &str);
    fn track_global_error(&mut self, error: &ErrorKind, error_context: &str);
    fn track_sent_bytes(&mut self, addr: &SocketAddr, bytes: usize);
    fn track_received_bytes(&mut self, addr: &SocketAddr, bytes: usize);
    fn track_ignored_bytes(&mut self, addr: &SocketAddr, bytes: usize);
    fn track_connection_destroyed(&mut self, addr: &SocketAddr);
}
