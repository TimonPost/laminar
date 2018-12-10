use super::Channel;

use crate::error::NetworkResult;
use crate::infrastructure::DeliveryMethod;
use crate::net::constants::STANDARD_HEADER_SIZE;
use crate::packet::header::{HeaderParser, HeaderReader, StandardHeader};
use crate::packet::{PacketData, PacketTypeId};

/// This channel should be used for unreliable processing of packets.
///
/// **Details**
///
/// | Packet Drop     | Packet Duplication | Packet Order     | Packet Fragmentation | Packet Delivery |
/// | :-------------: | :-------------:    | :-------------:  | :-------------:      | :-------------: |
/// |       Yes       |        Yes         |      No          |      No              |       No        |
///
/// Basically just bare UDP, free to be dropped, used for very unnecessary data, great for 'general' position updates.
/// Ordering depends on given 'ordering' parameter.
pub struct UnreliableChannel {
    ordered: bool,
}

impl UnreliableChannel {
    /// Create a new instance of the unreliable channel.
    pub fn new(ordered: bool) -> UnreliableChannel {
        UnreliableChannel { ordered }
    }

    /// Returns if a channel is ordered or not
    #[allow(dead_code)]
    pub fn is_ordered(&self) -> bool {
        self.ordered
    }
}

impl Channel for UnreliableChannel {
    /// This will prepossess an unreliable packet.
    ///
    /// 1. Generate default header.
    /// 2. Append payload.
    /// 3. Return the final data.
    fn process_outgoing(
        &mut self,
        payload: &[u8],
        delivery_method: DeliveryMethod,
    ) -> NetworkResult<PacketData> {
        let header = StandardHeader::new(delivery_method, PacketTypeId::Packet);
        let mut buffer = Vec::with_capacity(header.size() as usize);
        header.parse(&mut buffer)?;

        let mut packet_data = PacketData::with_capacity(payload.len());
        packet_data.add_fragment(&buffer, payload)?;
        Ok(packet_data)
    }

    /// Process a packet on receive.
    ///
    /// This will not do anything it will only return the bytes as they are received.
    fn process_incoming<'d>(&mut self, buffer: &'d [u8]) -> NetworkResult<&'d [u8]> {
        Ok(&buffer[STANDARD_HEADER_SIZE as usize..buffer.len()])
    }
}
