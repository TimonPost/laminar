use super::{HeaderReader, HeaderWriter};
use crate::error::Result;
use crate::net::constants::ACKED_PACKET_HEADER;
use crate::packet::header::StandardHeader;
use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use std::io::Cursor;

#[derive(Copy, Clone, Debug)]
/// This header providing reliability information.
pub struct AckedPacketHeader {
    /// StandardHeader for the Acked Packet
    pub standard_header: StandardHeader,
    /// this is the sequence number so that we can know where in the sequence of packages this packet belongs.
    pub seq: u16,
    // this is the last acknowledged sequence number.
    ack_seq: u16,
    // this is an bitfield of all last 32 acknowledged packages
    ack_field: u32,
}

impl AckedPacketHeader {
    /// When we compose packet headers, the local sequence becomes the sequence number of the packet, and the remote sequence becomes the ack.
    /// The ack bitfield is calculated by looking into a queue of up to 33 packets, containing sequence numbers in the range [remote sequence - 32, remote sequence].
    /// We set bit n (in [1,32]) in ack bits to 1 if the sequence number remote sequence - n is in the received queue.
    pub fn new(
        standard_header: StandardHeader,
        seq_num: u16,
        last_seq: u16,
        bit_field: u32,
    ) -> AckedPacketHeader {
        AckedPacketHeader {
            standard_header,
            seq: seq_num,
            ack_seq: last_seq,
            ack_field: bit_field,
        }
    }

    /// Get the sequence number from this packet.
    #[allow(dead_code)]
    pub fn sequence(&self) -> u16 {
        self.seq
    }

    /// Get bit field of all last 32 acknowledged packages
    pub fn ack_field(&self) -> u32 {
        self.ack_field
    }

    /// Get last acknowledged sequence number.
    pub fn ack_seq(&self) -> u16 {
        self.ack_seq
    }
}

impl HeaderWriter for AckedPacketHeader {
    type Output = Result<()>;

    fn parse(&self, buffer: &mut Vec<u8>) -> Self::Output {
        self.standard_header.parse(buffer)?;
        buffer.write_u16::<BigEndian>(self.seq)?;
        buffer.write_u16::<BigEndian>(self.ack_seq)?;
        buffer.write_u32::<BigEndian>(self.ack_field)?;
        Ok(())
    }
}

impl HeaderReader for AckedPacketHeader {
    type Header = Result<AckedPacketHeader>;

    fn read(rdr: &mut Cursor<&[u8]>) -> Self::Header {
        let standard_header = StandardHeader::read(rdr)?;
        let seq = rdr.read_u16::<BigEndian>()?;
        let ack_seq = rdr.read_u16::<BigEndian>()?;
        let ack_field = rdr.read_u32::<BigEndian>()?;

        Ok(AckedPacketHeader {
            standard_header,
            seq,
            ack_seq,
            ack_field,
        })
    }

    fn size(&self) -> u8 {
        ACKED_PACKET_HEADER
    }
}

#[cfg(test)]
mod tests {
    use crate::packet::header::{AckedPacketHeader, HeaderReader, HeaderWriter, StandardHeader};
    use std::io::Cursor;

    #[test]
    pub fn serializes_deserialize_acked_header_test() {
        let packet_header = AckedPacketHeader::new(StandardHeader::default(), 1, 1, 5421);
        let mut buffer = Vec::with_capacity((packet_header.size() + 1) as usize);

        let _ = packet_header.parse(&mut buffer);

        let mut cursor = Cursor::new(buffer.as_slice());

        match AckedPacketHeader::read(&mut cursor) {
            Ok(packet_deserialized) => {
                assert_eq!(packet_deserialized.seq, 1);
                assert_eq!(packet_deserialized.ack_seq, 1);
                assert_eq!(packet_deserialized.ack_field, 5421);
            }
            Err(e) => println!("{:?}", e),
        }
    }
}
