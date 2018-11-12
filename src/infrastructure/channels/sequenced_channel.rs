use super::Channel;

use error::NetworkResult;
use infrastructure::DeliveryMethod;
use packet::{PacketData, PacketTypeId};
use packet::header::{SequencedPacketHeader, StandardHeader, HeaderReader, HeaderParser};

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
#[derive(Default)]
pub struct SequencedChannel {
    seq_num: u16,
    remote_seq_num: u16,
}

impl SequencedChannel {
    /// Creates a new instance of the sequenced channel by specifying if channel needs to handle packets reliable.
    pub fn new() -> SequencedChannel {
        SequencedChannel { seq_num: 0, remote_seq_num: 0 }
    }
}

impl Channel for SequencedChannel {
    fn process_outgoing(
        &mut self,
        payload: &[u8],
        delivery_method: DeliveryMethod,
    ) -> NetworkResult<PacketData> {
        self.seq_num = self.seq_num.wrapping_add(1);

        let header = SequencedPacketHeader::new(StandardHeader::new(delivery_method, PacketTypeId::Packet), self.seq_num);
        let mut buffer = Vec::with_capacity(header.size() as usize);
        header.parse(&mut buffer)?;

        let mut packet_data = PacketData::with_capacity(buffer.len());
        packet_data.add_fragment(&buffer, payload)?;
        Ok(packet_data)
    }

    fn process_incoming<'d>(&mut self, buffer: &'d [u8]) -> NetworkResult<Option<&'d [u8]>> {
        let mut cursor = Cursor::new(buffer);
        let sequenced_header = SequencedPacketHeader::read(&mut cursor)?;

        if sequenced_header.sequence() > self.remote_seq_num {
            self.remote_seq_num = sequenced_header.sequence();
            Ok(Some(&buffer[sequenced_header.size() as usize..buffer.len()]))
        }
        else {
            Ok(None)
        }
    }
}
