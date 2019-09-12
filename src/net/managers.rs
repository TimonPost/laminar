pub use crate::either::Either;
use crate::net::events::ConnectionSendEvent;
use crate::packet::Packet;
use std::fmt::Debug;
use std::net::SocketAddr;

/// At any given moment, any connection can be only in these states.
/// These states are only managed through `ConnectionManager`, and define behaviour for sending and receiving packets.
/// Only these state transition is allowed:
/// | Old          | New          |
/// | ----------   | ----------   |
/// | Connecting   | Connected    |
/// | Connecting   | Disconnected |
/// | Connected    | Disconnected |
/// | Disconnected | Connecting   |
/// If these rules are not satisfied, panic! will be called.
/// Each state specifies what can and cannot be done:
/// * Connecting - This is initial state when socket is created, at this moment no `events` can be sent or received,
/// in this state `ConnectionManager` is able to receive and sent `packets` to properly initiate connection.
/// * Connected - Only in this state all events will be sent or received between peers.
/// * Disconnected - in this state `ConnectionManager` is not able to send or receive any packets.
/// It can only process incoming events and decide if it can reset connection and change to Connecting state,
/// otherwise connection will be closed when all packets-in-flight finishes sending or after connection timeout.
#[derive(Debug, PartialEq, Clone, Copy)]
pub enum ConnectionState {
    Connecting,
    Connected,
    Disconnected,
}

impl Default for ConnectionState {
    fn default() -> Self {
        Self::Connecting
    }
}

/// Generic error type, that is used by ConnectionManager implementation
#[derive(Debug, PartialEq, Clone)]
pub struct ConnectionManagerError(String);

/// It abstracts pure UDP packets, and allows to implement Connected/Disconnected states.
/// This table summary shows where exactly ConnectionManager sits in layer hierarchy.
/// | Abstraction layer | Capabilities                                                                                                                      |
/// |-------------------|-----------------------------------------------------------------------------------------------------------------------------------|
/// | Application       | Can receive these events: Created->Connected->Packet->Disconnected->Destroyed, And send these events: Connect->Packet->Disconnect |
/// | ConnectionManager | Receive raw Packet events, and manage connection state, can also generate Packets to initiate connection                          |
/// | Laminar           | Handles raw UDP bytes and provides reliability, ordering, fragmentation, etc.. capabilities                                       |
/// | ConnectionManager | May change raw incoming and outgoing bytes                                                                                        |
pub trait ConnectionManager: Debug {
    /// Pre-process incoming raw data, can be useful data needs to be decrypted before actually parsing packet
    fn preprocess_incoming<'s, 'a>(
        &'s self,
        data: &'a [u8],
    ) -> Result<Either<&'a [u8], Vec<u8>>, ConnectionManagerError>;
    /// Post-process outgoing data, can be useful to encrypt data before sending
    fn postprocess_outgoing<'s, 'a>(&'s self, data: &'a [u8]) -> Either<&'a [u8], Vec<u8>>;
    /// Process incoming packet:
    /// * can generate new packets for sending
    /// * always returns current state
    fn process_incoming(
        &mut self,
        packet: &Packet,
        createPacket: &mut dyn FnMut(Packet) -> Result<(), String>
    ) -> Result<ConnectionState, ConnectionManagerError>;
    /// Process outgoing packet:
    /// * can generate new packets for sending
    /// * always returns current state
    fn process_outgoing(
        &mut self, 
        event: &ConnectionSendEvent,
        createPacket: &mut dyn FnMut(Packet) -> Result<(), String>
    ) -> ConnectionState;
}

/// Tracks all sorts of global statistics, and decided whether to create `ConnectionManager` for new connections or not.
pub trait SocketManager: Debug {
    // answers very important question,
    // can we accept/create new connection, it also accepts raw bytes of the first packet, so it can do additional decision based on this
    fn accept_new_connection(
        &mut self,
        addr: &SocketAddr,
        bytes: &[u8],
    ) -> Option<Box<dyn ConnectionManager>>;

    // all sorts of statistics might be useful here to help deciding whether new connection can be created or not
    fn track_connection_error(&mut self, addr: &SocketAddr, error: String) {}
    fn track_sent_bytes(&mut self, addr: &SocketAddr, bytes: usize);
    fn track_received_bytes(&mut self, addr: &SocketAddr, bytes: usize);
    fn track_ignored_bytes(&mut self, addr: &SocketAddr, bytes: usize);
    fn track_connection_destroyed(&mut self, addr: &SocketAddr);
}

/// dumbies implementation that is always in connected state and simply resends all packets
#[derive(Debug)]
struct DumbConnectionManager;

impl ConnectionManager for DumbConnectionManager {
    // simply return same data
    fn preprocess_incoming<'s, 'a>(
        &'s self,
        data: &'a [u8],
    ) -> Result<Either<&'a [u8], Vec<u8>>, ConnectionManagerError> {
        Ok(Either::Left(data))
    }

    // simply return same data
    fn postprocess_outgoing<'s, 'a>(&'s self, data: &'a [u8]) -> Either<&'a [u8], Vec<u8>> {
        Either::Left(data)
    }

    fn process_incoming(
        &mut self,
        packet: &Packet,
        createPacket: &mut dyn FnMut(Packet) -> Result<(), String>
    ) -> Result<ConnectionState, ConnectionManagerError> {
        Ok(ConnectionState::Connected)
    }

    fn process_outgoing(
        &mut self,
        event: &ConnectionSendEvent,
        createPacket: &mut dyn FnMut(Packet) -> Result<(), String>
    ) -> ConnectionState {
        ConnectionState::Connected
    }
}

/// simples implementation of socket manager
/// it only does one thing, creates new connection only when he "properly" greets :)
#[derive(Debug)]
struct DumbSocketManager;

impl SocketManager for DumbSocketManager {
    fn accept_new_connection(
        &mut self,
        addr: &SocketAddr,
        bytes: &[u8],
    ) -> Option<Box<dyn ConnectionManager>> {
        if bytes == "Yo, man, its me!".as_bytes() {
            Some(Box::new(DumbConnectionManager))
        } else {
            println!("Ignore this address, until he properly greets;)");
            None
        }
    }

    fn track_connection_error(&mut self, addr: &SocketAddr, error: String) {}
    fn track_sent_bytes(&mut self, addr: &SocketAddr, bytes: usize) {}
    fn track_received_bytes(&mut self, addr: &SocketAddr, bytes: usize) {}
    fn track_ignored_bytes(&mut self, addr: &SocketAddr, bytes: usize) {}
    fn track_connection_destroyed(&mut self, addr: &SocketAddr) {}
}
