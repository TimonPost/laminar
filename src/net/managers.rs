pub use crate::either::Either;
pub use crate::net::events::{ConnectionClosedBy, DestroyReason};
use crate::packet::OutgoingPacket;
use crate::ErrorKind;
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
/// | Disconnected | Connecting   |
/// If these rules are not satisfied, panic! will be called.
/// Each state specifies what can and cannot be done:
/// * Connecting - This is initial state when socket is created, at this moment no `events` can be sent or received,
/// in this state `ConnectionManager` is able to receive and sent `packets` to properly initiate connection.
/// * Connected - Only in this state all events will be sent or received between peers.
/// * Disconnected - in this state `ConnectionManager` is not able to send or receive any packets.
/// It can only process incoming events and decide if it can reset connection and change to Connecting state,
/// otherwise connection will be closed when all packets-in-flight finishes sending or after connection timeout.
#[derive(Debug, PartialEq, Clone)]
pub enum ConnectionState {
    Connecting,
    Connected(Box<[u8]>),
    Disconnected(ConnectionClosedBy),
}

impl ConnectionState {
    /// Tries to change current state, returns old state if successfully changed.
    pub fn try_change(&mut self, new: Self) -> Option<Self> {
        match (&self, &new) {
            (ConnectionState::Connecting, ConnectionState::Connected(_))
            | (ConnectionState::Connecting, ConnectionState::Disconnected(_))
            | (ConnectionState::Connected(_), ConnectionState::Disconnected(_))
            | (ConnectionState::Disconnected(_), ConnectionState::Connecting) => {
                Some(std::mem::replace(self, new))
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

/// Generic error type, that is used by ConnectionManager implementation
#[derive(Debug, PartialEq, Clone)]
pub struct ConnectionManagerError(String);

/// It abstracts pure UDP packets, and allows to implement Connected/Disconnected states.
/// This table summary shows where exactly ConnectionManager sits in between different layers.
/// | Abstraction layer | Capabilities                                                                                                                                                        |
/// |-------------------|---------------------------------------------------------------------------------------------------------------------------------------------------------------------|
/// | Application       | Can receive these events: Created->Connected(data)->Packet(data)->Disconnected(reason)->Destroyed(reason) Can send these events: Connect(data)->Packet->Disconnect. |
/// | ConnectionManager | Receive incoming and outgoing packets and manage connection state. Can generate new packets to initiate connection.                                                 |
/// | Laminar           | Adds/Removes headers to packets, so that it could provides reliability, ordering, fragmentation, etc.. capabilities.                                                |
/// | ConnectionManager | May change raw incoming and outgoing bytes to apply encryption, compression, etc.                                                                                   |
///
/// Manager can store local buffer size of `Config.receive_buffer_max_size` if it does some kind of encryption.
/// It tries to maintain valid connection state, and it can't decide when to destroy or disconnect a connection itself.
/// Only when packet is recevied, or action is initiated by user it is allowed to change connection state.
/// From the point of view of connection manager, laminar's header + payload is interpreted as user data.
/// Distinction between user packet and protocol specific packet is encoded in laminar's packet header.
pub trait ConnectionManager: Debug {
    /// This function should be called frequently, even if there is no packets to send or receive.
    /// It will always be called last, after all other methods is called, so it could send packets or comunicate errors if required.
    /// It cannot change connection state explicitly, instead it can emit errors, and SocketManager will decide when to destroy connection.
    /// It can generate all kinds of packets: heartbeat, user or connection protocol packets.
    /// (maybe heartbeat functionality should be moved here?)
    /// `buffer` is used to return packet payload, it's size is `Config.receive_buffer_max_size`.
    fn update<'a>(
        &mut self,
        buffer: &'a mut [u8],
        time: Instant,
    ) -> Option<Result<Either<OutgoingPacket<'a>, &ConnectionState>, ConnectionManagerError>>;

    /// This will be called for all incoming data, including packets that were resent by remote host.
    /// If packet is accepted by laminar's reliability layer `process_protocol_data` will be called immediatelly.
    /// It should return a slice where header + payload exists
    fn preprocess_incoming<'a, 'b>(
        &mut self,
        data: &'a [u8],
        buffer: &'b mut [u8],
    ) -> Result<&'b [u8], ConnectionManagerError>
    where
        'a: 'b;

    /// This will be called for all outgoing data, including packets that are resend.    
    /// Dropped packets will also go through here.
    /// Accepts full packet: header + payload
    fn postprocess_outgoing<'a, 'b>(&mut self, data: &'a [u8], buffer: &'b mut [u8]) -> &'b [u8]
    where
        'a: 'b;

    /// This will be called only for incoming protocol specific packets, after laminar's reliability layer accepted it.
    /// This is the only place where connection state can actually be changed by incomming packet.
    fn process_protocol_data<'a>(
        &mut self,
        data: &'a [u8],
    ) -> Result<&ConnectionState, ConnectionManagerError>;

    /// This will be invoked when player sends connect request,
    /// Some protocols might provide a way to pass initial connection data
    /// Data is user payload, that will be received by remote host, on Connected(data) event
    /// This method is not able to send packet immediatelly, instead this functionality should be handled in `update` method.
    fn connect<'a>(&mut self, data: &'a [u8]);

    // This will be invoked when player sends disconnect request,
    /// This method is not able to send packet immediatelly, instead this functionality should be handled in `update` method.
    fn disconnect<'a>(&mut self);
}

/// Tracks all sorts of global statistics, and decided whether to create `ConnectionManager` for new connections or not.
/// Also decides when connections should be destroyed even if they are in connected state.
pub trait SocketManager: Debug {
    /// Decide if it is possible to accept/create new connection connection
    fn accept_new_connection(&mut self, addr: &SocketAddr) -> Option<Box<dyn ConnectionManager>>;

    /// Returns list of connections that socket manager decided to destroy
    fn destroy_connections(&mut self) -> Option<Vec<(SocketAddr, DestroyReason)>>;

    // all sorts of statistics might be useful here to help deciding whether new connection can be created or not
    fn track_connection_error(&mut self, addr: &SocketAddr, error: &ErrorKind);
    fn track_global_error(&mut self, error: &ErrorKind);
    fn track_sent_bytes(&mut self, addr: &SocketAddr, bytes: usize);
    fn track_received_bytes(&mut self, addr: &SocketAddr, bytes: usize);
    fn track_ignored_bytes(&mut self, addr: &SocketAddr, bytes: usize);
    fn track_connection_destroyed(&mut self, addr: &SocketAddr);
}

/// Simple connection manager, sends "connect" and "disconnect" messages and changes states when receive either of theses messages
#[derive(Debug, Default)]
struct SimpleConnectionManager {
    state: ConnectionState,
    send: Option<Box<[u8]>>,
}

use crate::packet::{DeliveryGuarantee, OrderingGuarantee, OutgoingPacketBuilder, PacketType};

impl ConnectionManager for SimpleConnectionManager {
    fn update<'a>(
        &mut self,
        buffer: &'a mut [u8],
        time: Instant,
    ) -> Option<Result<Either<OutgoingPacket<'a>, &ConnectionState>, ConnectionManagerError>> {
        match self.send.take() {
            Some(data) => {
                // copy from buffer what we want to send
                buffer.copy_from_slice(data.as_ref());
                // create packet and set packet type ConnectionManager, so that it can be processed by `process_protocol_data` in remote host
                Some(Ok(Either::Left(
                    OutgoingPacketBuilder::new(buffer)
                        .with_default_header(
                            PacketType::ConnectionManager,
                            DeliveryGuarantee::Reliable,
                            OrderingGuarantee::Ordered(None),
                        )
                        .build(),
                )))
            }
            None => None,
        }
    }

    fn preprocess_incoming<'a, 'b>(
        &mut self,
        data: &'a [u8],
        _buffer: &'b mut [u8],
    ) -> Result<&'b [u8], ConnectionManagerError>
    where
        'a: 'b,
    {
        Ok(data)
    }

    fn postprocess_outgoing<'a, 'b>(&mut self, data: &'a [u8], _buffer: &'b mut [u8]) -> &'b [u8]
    where
        'a: 'b,
    {
        data
    }

    fn process_protocol_data<'a>(
        &mut self,
        data: &'a [u8],
    ) -> Result<&ConnectionState, ConnectionManagerError> {
        if data.starts_with("connect".as_bytes()) {
            self.state
                .try_change(ConnectionState::Connected(Box::from(data.split_at(7).1)));
        } else if data.eq("disconnect".as_bytes()) {
            self.state.try_change(ConnectionState::Disconnected(ConnectionClosedBy::RemoteHost));
        } else {
            return Err(ConnectionManagerError(format!(
                "Unknown message type: {:?}",
                std::str::from_utf8(data)
            )));
        }
        Ok(&self.state)
    }

    fn connect<'a>(&mut self, data: &'a [u8]) {
        self.send = Some(Box::from(["connect".as_bytes(), data].concat()));
    }

    fn disconnect<'a>(&mut self) {
        self.state.try_change(ConnectionState::Disconnected(ConnectionClosedBy::LocalHost));
        self.send = Some(Box::from("disconnect".as_bytes()));
    }
}

/// Simplest implementation of socket manager, always accept connection and never destroy, no matter how many error connection can report
#[derive(Debug)]
struct DumbSocketManager;

impl SocketManager for DumbSocketManager {
    fn accept_new_connection(&mut self, addr: &SocketAddr) -> Option<Box<dyn ConnectionManager>> {
        Some(Box::new(SimpleConnectionManager::default()))
    }

    fn destroy_connections(&mut self) -> Option<Vec<(SocketAddr, DestroyReason)>> {
        None
    }
    
    fn track_connection_error(&mut self, addr: &SocketAddr, error: &ErrorKind) {}
    fn track_global_error(&mut self, error: &ErrorKind) {}
    fn track_sent_bytes(&mut self, addr: &SocketAddr, bytes: usize) {}
    fn track_received_bytes(&mut self, addr: &SocketAddr, bytes: usize) {}
    fn track_ignored_bytes(&mut self, addr: &SocketAddr, bytes: usize) {}
    fn track_connection_destroyed(&mut self, addr: &SocketAddr) {}
}
