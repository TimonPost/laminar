use super::Channel;

use packet::PacketData;
use infrastructure::DeliveryMethod;
use error::NetworkResult;

use std::io::Cursor;

/// This channel should be used for processing packets sequenced.
///
/// *Details*
///
/// |   Packet Drop   | Packet Duplication | Packet Order     | Packet Fragmentation | Packet Delivery |
/// | :-------------: | :-------------:    | :-------------:  | :-------------:      | :-------------: |
/// |       Yes       |      No            |      Yes         |      Yes             |       No        |
///
/// Toss away any packets that are older than the most recent (like a position update, you don't care about older ones),
/// packets may be dropped, just the application may not receive older ones if a newer one came in first.
/// This
pub struct SequencedChannel;

impl SequencedChannel {
    /// Creates a new instance of the sequenced channel by specifying if channel needs to handle packets reliable.
    pub fn new() -> SequencedChannel {
        SequencedChannel
    }
}

impl Channel for SequencedChannel {
    fn process_outgoing(&mut self, payload: &[u8], delivery_method: DeliveryMethod) -> NetworkResult<PacketData> {
        unimplemented!()
    }

    fn process_incoming<'d>(&mut self, buffer: &'d[u8]) -> NetworkResult<&'d[u8]> {
        unimplemented!()
    }
}