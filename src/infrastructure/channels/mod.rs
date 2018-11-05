//! This module provides channels for processing packets of different reliabilities.

mod unreliable_channel;
mod sequenced_channel;
mod reliable_channel;

use infrastructure::DeliveryMethod;
use error::NetworkResult;
use packet::PacketData;
use std::io::Cursor;

pub use self::unreliable_channel::UnreliableChannel;
pub use self::sequenced_channel::SequencedChannel;
pub use self::reliable_channel::ReliableChannel;

/// This provides an abstraction for processing packets to their given reliability.
pub trait Channel {
    // Process a packet before sending it and return a packet instance with the given raw data.
    fn process_outgoing(&mut self, payload: &[u8], delivery_method: DeliveryMethod) -> NetworkResult<PacketData>;

    /// Progress an packet on receive and receive the processed data.
    fn process_incoming<'d>(&mut self, buffer: &'d[u8]) -> NetworkResult<&'d[u8]>;
}

