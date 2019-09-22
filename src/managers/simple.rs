use crate::net::managers::*;

use crate::packet::{DeliveryGuarantee, OrderingGuarantee};
use std::collections::VecDeque;
use std::net::SocketAddr;
use std::time::Instant;

/// The simplest connection manager, that immediately goes into the connected state, after creating it
#[derive(Debug)]
struct AlwaysConnectedConnectionManager {
    // this is used to set initial state as connected when creating an instance.
    // we'll take this value on first `update` call
    initial_state: Option<ConnectionState>,
}

impl Default for AlwaysConnectedConnectionManager {
    fn default() -> Self {
        Self {
            // initialize to connected state on creation.
            initial_state: Some(ConnectionState::Connected(Box::default())),
        }
    }
}

impl ConnectionManager for AlwaysConnectedConnectionManager {
    fn update<'a>(
        &mut self,
        _buffer: &'a mut [u8],
        _time: Instant,
    ) -> Option<ConnectionManagerEvent<'a>> {
        self.initial_state
            .take() // on first call state will be moved out.
            .map(ConnectionManagerEvent::NewState)
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

    fn process_protocol_data(&mut self, _data: &[u8]) -> Result<(), ConnectionManagerError> {
        Ok(())
    }

    fn connect(&mut self, _data: Box<[u8]>) {}

    fn disconnect(&mut self) {}
}

/// Simple connection manager, that actually tries to connect by exchanging 'connect', 'connected', and 'disconnect' messages with the remote host,
#[derive(Debug, Default)]
struct SimpleConnectionManager {
    state: ConnectionState,
    changes: VecDeque<Either<Box<[u8]>, ConnectionState>>,
}

impl SimpleConnectionManager {
    fn change_state(&mut self, new: ConnectionState) {
        if self.state.try_change(&new).is_some() {
            self.changes.push_back(Either::Right(self.state.clone()));
        }
    }

    fn send_packet(&mut self, payload: &[u8]) {
        self.changes.push_back(Either::Left(Box::from(payload)));
    }

    fn get_packet<'a>(data: Box<[u8]>, buffer: &'a mut [u8]) -> GenericPacket<'a> {
        // get result slice
        let payload = &mut buffer[0..data.as_ref().len()];
        // copy from buffer what we want to send
        payload.copy_from_slice(data.as_ref());
        // create packet
        GenericPacket::connection_packet(
            payload,
            DeliveryGuarantee::Reliable,
            OrderingGuarantee::None,
        )
    }
}

impl ConnectionManager for SimpleConnectionManager {
    fn update<'a>(
        &mut self,
        buffer: &'a mut [u8],
        _time: Instant,
    ) -> Option<ConnectionManagerEvent<'a>> {
        self.changes
            .pop_front()
            .take()
            .map(move |event| match event {
                Either::Left(data) => ConnectionManagerEvent::NewPacket(
                    SimpleConnectionManager::get_packet(data, buffer),
                ),
                Either::Right(state) => ConnectionManagerEvent::NewState(state),
            })
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

    fn process_protocol_data(&mut self, data: &[u8]) -> Result<(), ConnectionManagerError> {
        match self.state {
            ConnectionState::Connecting => {
                if data.starts_with(b"connect-") {
                    self.send_packet(b"connected-");
                    self.change_state(ConnectionState::Connected(Box::from(data.split_at(8).1)));
                } else if data.starts_with(b"connected-") {
                    self.change_state(ConnectionState::Connected(Box::from(data.split_at(10).1)));
                }
            }
            ConnectionState::Connected(_) => {
                if data.eq(b"disconnect-") {
                    self.change_state(ConnectionState::Disconnected(TargetHost::RemoteHost));
                }
            }
            _ => panic!("In disconnected nothing can happen"),
        }
        Ok(())
    }

    fn connect(&mut self, data: Box<[u8]>) {
        self.send_packet([b"connect-", data.as_ref()].concat().as_ref());
    }

    fn disconnect(&mut self) {
        if let ConnectionState::Connected(_) = self.state {
            self.send_packet(b"disconnect-");
        }
        self.change_state(ConnectionState::Disconnected(TargetHost::LocalHost));
    }
}

/// Simplest implementation of socket manager, always accept a connection and never destroy, no matter how many errors connection reports
/// It can create two types of connection managers:
/// * true - creates `AlwaysConnectedConnectionManager`
/// * false - creates `SimpleConnectionManager`
#[derive(Debug)]
pub struct SimpleConnectionManagerFactory(pub bool);

impl ConnectionManagerFactory for SimpleConnectionManagerFactory {
    fn create_remote_connection_manager(
        &mut self,
        addr: &SocketAddr,
        _raw_bytes: &[u8],
    ) -> Box<dyn ConnectionManager> {
        self.create_local_connection_manager(addr)
    }

    fn create_local_connection_manager(
        &mut self,
        _addr: &SocketAddr,
    ) -> Box<dyn ConnectionManager> {
        if self.0 {
            Box::new(AlwaysConnectedConnectionManager::default())
        } else {
            Box::new(SimpleConnectionManager::default())
        }
    }
}
