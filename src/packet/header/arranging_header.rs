use super::{HeaderReader, HeaderWriter};
use crate::error::Result;
use crate::net::constants::ARRANGING_PACKET_HEADER;
use crate::packet::SequenceNumber;
use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use std::io::Cursor;

#[derive(Copy, Clone, Debug)]
/// This header represents a fragmented packet header.
pub struct ArrangingHeader {
    arranging_id: SequenceNumber,
    stream_id: u8,
}

impl ArrangingHeader {
    /// Create new fragment with the given packet header
    pub fn new(arranging_id: SequenceNumber, stream_id: u8) -> Self {
        ArrangingHeader {
            arranging_id,
            stream_id,
        }
    }

    /// Get the sequence number from this packet.
    pub fn arranging_id(&self) -> SequenceNumber {
        self.arranging_id
    }

    /// Get the sequence number from this packet.
    pub fn stream_id(&self) -> u8 {
        self.stream_id
    }
}

impl HeaderWriter for ArrangingHeader {
    type Output = Result<()>;

    fn parse(&self, buffer: &mut Vec<u8>) -> Self::Output {
        buffer.write_u16::<BigEndian>(self.arranging_id)?;
        buffer.write_u8(self.stream_id)?;

        Ok(())
    }
}

impl HeaderReader for ArrangingHeader {
    type Header = Result<ArrangingHeader>;

    fn read(rdr: &mut Cursor<&[u8]>) -> Self::Header {
        let arranging_id = rdr.read_u16::<BigEndian>()?;
        let stream_id = rdr.read_u8()?;

        let header = ArrangingHeader {
            arranging_id,
            stream_id,
        };

        Ok(header)
    }

    /// Get the size of this header.
    fn size() -> u8 {
        ARRANGING_PACKET_HEADER
    }
}

#[cfg(test)]
mod tests {
    use crate::net::constants::ARRANGING_PACKET_HEADER;
    use crate::packet::header::{ArrangingHeader, HeaderReader, HeaderWriter};
    use std::io::Cursor;

    #[test]
    fn serialize() {
        let mut buffer = Vec::new();
        let header = ArrangingHeader::new(1, 2);
        header.parse(&mut buffer).is_ok();

        assert_eq!(buffer[1], 1);
        assert_eq!(buffer[2], 2);
    }

    #[test]
    fn deserialize() {
        let buffer = vec![0, 1, 2];
        let mut cursor = Cursor::new(buffer.as_slice());

        let header = ArrangingHeader::read(&mut cursor).unwrap();

        assert_eq!(header.arranging_id(), 1);
        assert_eq!(header.stream_id(), 2);
    }

    #[test]
    fn size() {
        assert_eq!(ArrangingHeader::size(), ARRANGING_PACKET_HEADER);
    }
}
