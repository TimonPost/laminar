use super::{HeaderParser, HeaderReader};
use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use error::NetworkResult;
use net::constants::SEQUENCED_PACKET_HEADER;
use packet::header::StandardHeader;
use std::io::Cursor;

#[derive(Copy, Clone, Debug)]
/// This header providing reliability information.
pub struct SequencedPacketHeader {
    /// StandardHeader for the Acked Packet
    pub standard_header: StandardHeader,
    pub seq: u16,
}

impl SequencedPacketHeader {
    /// When we compose packet headers, the local sequence becomes the sequence number of the packet, and the remote sequence becomes the ack.
    /// The ack bitfield is calculated by looking into a queue of up to 33 packets, containing sequence numbers in the range [remote sequence - 32, remote sequence].
    /// We set bit n (in [1,32]) in ack bits to 1 if the sequence number remote sequence - n is in the received queue.
    pub fn new(
        standard_header: StandardHeader,
        seq_num: u16
    ) -> SequencedPacketHeader {
        SequencedPacketHeader {
            standard_header,
            seq: seq_num,
        }
    }

    /// Get the sequence number from this packet.
    #[allow(dead_code)]
    pub fn sequence(&self) -> u16 {
        self.seq
    }
}

impl HeaderParser for SequencedPacketHeader {
    type Output = NetworkResult<()>;

    fn parse(&self, buffer: &mut Vec<u8>) -> <Self as HeaderParser>::Output {
        self.standard_header.parse(buffer)?;
        buffer.write_u16::<BigEndian>(self.seq)?;
        Ok(())
    }
}

impl HeaderReader for SequencedPacketHeader {
    type Header = NetworkResult<SequencedPacketHeader>;

    fn read(rdr: &mut Cursor<&[u8]>) -> <Self as HeaderReader>::Header {
        let standard_header = StandardHeader::read(rdr)?;
        let seq = rdr.read_u16::<BigEndian>()?;

        Ok(SequencedPacketHeader {
            standard_header,
            seq
        })
    }

    fn size(&self) -> u8 {
        SEQUENCED_PACKET_HEADER
    }
}

#[cfg(test)]
mod tests {
    use packet::header::{SequencedPacketHeader, HeaderParser, HeaderReader, StandardHeader};
    use std::io::Cursor;

    #[test]
    pub fn serializes_deserialize_acked_header_test() {
        let packet_header = SequencedPacketHeader::new(StandardHeader::default(), 1);
        let mut buffer = Vec::with_capacity((packet_header.size() + 1) as usize);

        let _ = packet_header.parse(&mut buffer);

        let mut cursor = Cursor::new(buffer.as_slice());

        match SequencedPacketHeader::read(&mut cursor) {
            Ok(packet_deserialized) => {
                assert_eq!(packet_deserialized.seq, 1);
            }
            Err(e) => println!("{:?}", e),
        }
    }
}
