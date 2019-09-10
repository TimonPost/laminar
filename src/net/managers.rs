pub use crate::either::Either;
use crate::net::events::ConnectionSendEvent;
use crate::packet::Packet;
use std::fmt::Debug;
use std::net::SocketAddr;

#[derive(Debug)]
pub enum ConnectionManagerState {
    Connecting,
    Connected,
    Disconnected,
}

pub enum PacketsToSend<'a> {
    None,
    Same(&'a Packet),
    New(Vec<Packet>),
}

pub struct ProcessPacketResult<'a> {
    pub packets: PacketsToSend<'a>,
    pub state: ConnectionManagerState,
}

impl<'a> ProcessPacketResult<'a> {
    pub fn new(packets: PacketsToSend<'a>, state: ConnectionManagerState) -> Self {
        Self {
            packets,
            state,
        }
    }
}

/// This is higher level abstraction allows to implement encrypt, custom authentication schemes, etc.
/// It can initiate handshaking process
/// Client and host messages will only passthrough when connection is in Connected state
pub trait ConnectionManager: Debug {
    /// Pre-process incoming raw data, can be useful data needs to be decrypted before actually parsing packet
    fn preprocess_incoming<'s, 'a>(
        &'s self,
        data: &'a [u8],
    ) -> Result<Either<&'a [u8], Vec<u8>>, String>;
    /// Post-process outgoing data, can be useful to encrypt data before sending
    fn postprocess_outgoing<'s, 'a>(&'s self, data: &'a [u8]) -> Either<&'a [u8], Vec<u8>>;
    /// Process incoming packet:
    /// * can generate new packets for sending
    /// * always returns current state
    fn process_incoming<'s, 'a>(
        &'s mut self,
        packet: &'a Packet,
    ) -> Result<ProcessPacketResult<'a>, String>;
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
    ) -> Result<Either<&'a [u8], Vec<u8>>, String> {
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
    ) -> Result<ProcessPacketResult<'a>, String> {
        Ok(ProcessPacketResult::new(
            PacketsToSend::None,
            ConnectionManagerState::Connected,
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
                ConnectionManagerState::Connected,
            ),
            _ => ProcessPacketResult::new(PacketsToSend::None, ConnectionManagerState::Connected),
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
