use super::{HeaderReader, HeaderWriter};
use crate::error::Result;
use crate::net::constants::ACKED_PACKET_HEADER;
use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use std::io::Cursor;

#[derive(Copy, Clone, Debug)]
/// This header providing reliability information.
pub struct AckedPacketHeader {
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
    pub fn new(seq_num: u16, last_seq: u16, bit_field: u32) -> AckedPacketHeader {
        AckedPacketHeader {
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
        buffer.write_u16::<BigEndian>(self.seq)?;
        buffer.write_u16::<BigEndian>(self.ack_seq)?;
        buffer.write_u32::<BigEndian>(self.ack_field)?;
        Ok(())
    }
}

impl HeaderReader for AckedPacketHeader {
    type Header = Result<AckedPacketHeader>;

    fn read(rdr: &mut Cursor<&[u8]>) -> Self::Header {
        let seq = rdr.read_u16::<BigEndian>()?;
        let ack_seq = rdr.read_u16::<BigEndian>()?;
        let ack_field = rdr.read_u32::<BigEndian>()?;

        Ok(AckedPacketHeader {
            seq,
            ack_seq,
            ack_field,
        })
    }

    fn size() -> u8 {
        ACKED_PACKET_HEADER
    }
}

#[cfg(test)]
mod tests {
    use crate::net::constants::ACKED_PACKET_HEADER;
    use crate::packet::header::{AckedPacketHeader, HeaderReader, HeaderWriter};
    use std::io::Cursor;

    #[test]
    fn serialize() {
        let mut buffer = Vec::new();
        let header = AckedPacketHeader::new(1, 2, 3);
        header.parse(&mut buffer).is_ok();

        assert_eq!(buffer[1], 1);
        assert_eq!(buffer[3], 2);
        assert_eq!(buffer[7], 3);
        assert_eq!(buffer.len() as u8, AckedPacketHeader::size());
    }

    #[test]
    fn deserialize() {
        let buffer = vec![0, 1, 0, 2, 0, 0, 0, 3];

        let mut cursor = Cursor::new(buffer.as_slice());

        let header = AckedPacketHeader::read(&mut cursor).unwrap();

        assert_eq!(header.sequence(), 1);
        assert_eq!(header.ack_seq(), 2);
        assert_eq!(header.ack_field(), 3);
    }

    #[test]
    fn size() {
        assert_eq!(AckedPacketHeader::size(), ACKED_PACKET_HEADER);
    }
}
