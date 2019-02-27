//! This module provides channels for processing packets of different reliabilities.

mod reliable_channel;
mod sequenced_channel;
mod unreliable_channel;

use crate::error::Result;
use crate::infrastructure::DeliveryMethod;
use crate::packet::PacketData;

pub use self::reliable_channel::ReliableChannel;
pub use self::sequenced_channel::SequencedChannel;
pub use self::unreliable_channel::UnreliableChannel;

/// This provides an abstraction for processing packets to their given reliability.
pub trait Channel {
    /// Process a packet before sending it and return a packet instance with the given raw data.
    fn process_outgoing(
        &mut self,
        payload: &[u8],
        delivery_method: DeliveryMethod,
    ) -> Result<PacketData>;

    /// Progress an packet on receive and receive the processed data.
    fn process_incoming<'d>(&mut self, buffer: &'d [u8]) -> Result<&'d [u8]>;
}
