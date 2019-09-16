use crate::net::managers::*;

use crate::packet::{DeliveryGuarantee, OrderingGuarantee, OutgoingPacketBuilder, PacketType};
use log::error;
use std::io::ErrorKind::WouldBlock;
use std::net::SocketAddr;
use std::time::Instant;
use std::collections::VecDeque;

/// Simple connection manager, sends "connect" and "disconnect" messages and changes states when receive either of these messages
#[derive(Debug, Default)]
struct SimpleConnectionManager {
    state: ConnectionState,
    changes: VecDeque<Either<Box<[u8]>, ConnectionState>>,
}

impl SimpleConnectionManager {
    fn change_state(&mut self, new: ConnectionState) {
        if let Some(_) = self.state.try_change(&new) {
            self.changes.push_back(Either::Right(self.state.clone()));
        }
    }

    fn send_packet(&mut self, payload:&[u8]) {
        self.changes.push_back(Either::Left(Box::from(payload)));
    }

    fn get_packet<'a> (data: Box<[u8]>, buffer: &'a mut [u8]) -> OutgoingPacket<'a> {
        // get result slice
        let res_buffer = &mut buffer[0..data.as_ref().len()];                        
        // copy from buffer what we want to send
        res_buffer.copy_from_slice(data.as_ref());
        println!("===========================update send {}", String::from_utf8_lossy(res_buffer));
        // create packet
        OutgoingPacketBuilder::new(res_buffer)
        .with_default_header(
            PacketType::ConnectionManager,
            DeliveryGuarantee::Unreliable,
            OrderingGuarantee::None,
        )
        .build()
    }
}

impl ConnectionManager for SimpleConnectionManager {
    fn update<'a>(
        &mut self,
        buffer: &'a mut [u8],
        _time: Instant,
    ) -> Option<Result<Either<OutgoingPacket<'a>, ConnectionState>, ConnectionManagerError>> {        
        match self.changes.pop_front().take() {
            Some(change) => {
                Some(Ok(match change {
                    Either::Left(data) => Either::Left(SimpleConnectionManager::get_packet(data, buffer)),
                    Either::Right(state) => Either::Right(state)
                }))
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
        println!(
            "===========================preprocess_incoming: {}",
            String::from_utf8_lossy(data)
        );
        Ok(data)
    }

    fn postprocess_outgoing<'a, 'b>(&mut self, data: &'a [u8], _buffer: &'b mut [u8]) -> &'b [u8]
    where
        'a: 'b,
    {
        println!(
            "===========================postprocess_outgoing: {}",
            String::from_utf8_lossy(data)
        );
        data
    }

    fn process_protocol_data<'a>(&mut self, data: &'a [u8]) -> Result<(), ConnectionManagerError> {
        println!(
            "===========================process_protocol_data: {}",
            String::from_utf8_lossy(data)
        );
        if data.starts_with("connect-".as_bytes()) {
            self.change_state(ConnectionState::Connected(Box::from(data.split_at(8).1)));
            self.send_packet("connected-".as_bytes());
        } else if data.starts_with("connected-".as_bytes()) {
            self.change_state(ConnectionState::Connected(Box::from(data.split_at(10).1)));
        } else if data.starts_with("disconnect".as_bytes()) {
            self.change_state(ConnectionState::Disconnected(TargetHost::RemoteHost));
        } else {
            return Err(ConnectionManagerError(format!(
                "Unknown message type: {:?}",
                String::from_utf8_lossy(data)
            )));
        }
        Ok(())
    }

    fn connect<'a>(&mut self, data: Box<[u8]>) {
        self.send_packet(["connect-".as_bytes(), data.as_ref()].concat().as_ref());
    }

    fn disconnect<'a>(&mut self) {
        self.change_state(ConnectionState::Disconnected(TargetHost::LocalHost));
        self.send_packet("disconnect".as_bytes());
    }

}

/// Simplest implementation of socket manager, always accept connection and never destroy, no matter how many errors connection reports
#[derive(Debug)]
pub struct SimpleSocketManager;

impl SocketManager for SimpleSocketManager {
    fn accept_new_connection(
        &mut self,
        _addr: &SocketAddr,
        _requested_by: TargetHost,
    ) -> Option<Box<dyn ConnectionManager>> {
        Some(Box::new(SimpleConnectionManager::default()))
    }

    fn collect_connections_to_destroy(&mut self) -> Option<Vec<(SocketAddr, DestroyReason)>> {
        None
    }

    fn track_connection_error(
        &mut self,
        addr: &SocketAddr,
        error: &ErrorKind,
        error_context: &str,
    ) {
        match error {
            ErrorKind::IOError(ref e) if e.kind() == WouldBlock => {}
            _ => error!("Error, {} ({:?}): {:?}", error_context, addr, error),
        }
    }
    fn track_global_error(&mut self, error: &ErrorKind, error_context: &str) {
        error!("Error, {}: {:?}", error_context, error);
    }
    fn track_sent_bytes(&mut self, _addr: &SocketAddr, _bytes: usize) {}
    fn track_received_bytes(&mut self, _addr: &SocketAddr, _bytes: usize) {}
    fn track_ignored_bytes(&mut self, _addr: &SocketAddr, _bytes: usize) {}
    fn track_connection_destroyed(&mut self, _addr: &SocketAddr) {}
}
