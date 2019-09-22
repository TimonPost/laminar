use crate::error::ErrorKind;

use log::error;
use std::io::ErrorKind::WouldBlock;
use std::net::SocketAddr;

/// Tracks all sorts of global statistics
// TODO write implementation of this
#[derive(Debug)]
pub struct MetricsCollector {}

impl MetricsCollector {
    pub fn track_connection_error(
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
    pub fn track_global_error(&mut self, error: &ErrorKind, error_context: &str) {
        error!("Error, {}: {:?}", error_context, error);
    }

    pub fn track_sent_bytes(&mut self, _addr: &SocketAddr, _bytes: usize) {}
    pub fn track_received_bytes(&mut self, _addr: &SocketAddr, _bytes: usize) {}
    pub fn track_ignored_bytes(&mut self, _addr: &SocketAddr, _bytes: usize) {}

    pub fn track_connection_created(&mut self, _addr: &SocketAddr) {}

    pub fn track_connection_destroyed(&mut self, _addr: &SocketAddr) {}
}
