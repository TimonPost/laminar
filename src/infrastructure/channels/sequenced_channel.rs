use super::Channel;

use crate::error::Result;
use crate::infrastructure::DeliveryMethod;
use crate::packet::PacketData;

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

#[derive(Default)]
pub struct SequencedChannel;

impl SequencedChannel {
    /// Creates a new instance of the sequenced channel by specifying if channel needs to handle packets reliable.
    pub fn new() -> SequencedChannel {
        SequencedChannel
    }
}

impl Channel for SequencedChannel {
    fn process_outgoing(
        &mut self,
        _payload: &[u8],
        _delivery_method: DeliveryMethod,
    ) -> Result<PacketData> {
        unimplemented!()
    }

    fn process_incoming<'d>(&mut self, _buffer: &'d [u8]) -> Result<&'d [u8]> {
        unimplemented!()
    }
}
