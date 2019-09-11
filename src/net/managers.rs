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
#[derive(Debug, PartialEq)]
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

pub enum PacketsToSend<'a> {
    None,
    Same(&'a Packet),
    New(Vec<Packet>),
}

pub struct ProcessPacketResult<'a> {
    pub packets: PacketsToSend<'a>,
    pub state: ConnectionState,
}

impl<'a> ProcessPacketResult<'a> {
    pub fn new(packets: PacketsToSend<'a>, state: ConnectionState) -> Self {
        Self {
            packets,
            state,
        }
    }
}

#[derive(Debug)]
pub enum ConnectionManagerError {
    UnableToPreprocess(String),
    HandshakeError(String)
}

/// This is higher level abstraction allows to implement encrypt, custom authentication schemes, etc.
/// It can initiate handshaking process
/// Client and host messages will only passthrough when connection is in Connected state
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
    fn process_incoming<'s, 'a>(
        &'s mut self,
        packet: &'a Packet,
    ) -> Result<ProcessPacketResult<'a>, ConnectionManagerError>;
    /// Process incoming packet:
    /// * can generate new packets for sending
    /// * always returns current state
    fn process_outgoing<'s, 'a>(
        &'s mut self,
        event: &'a ConnectionSendEvent,
    ) -> ProcessPacketResult<'a>;
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

    // this forwards all packets and always stays in connected state
    fn process_incoming<'s, 'a>(
        &'s mut self,
        packet: &'a Packet,
    ) -> Result<ProcessPacketResult<'a>, ConnectionManagerError> {
        Ok(ProcessPacketResult::new(
            PacketsToSend::None,
            ConnectionState::Connected,
        ))
    }
    // ignore connect and disconnect events, and simply forward the data
    fn process_outgoing<'s, 'a>(
        &'s mut self,
        event: &'a ConnectionSendEvent,
    ) -> ProcessPacketResult<'a> {
        match &event {
            ConnectionSendEvent::Packet(data) => ProcessPacketResult::new(
                PacketsToSend::Same(data),
                ConnectionState::Connected,
            ),
            _ => ProcessPacketResult::new(PacketsToSend::None, ConnectionState::Connected),
        }
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
