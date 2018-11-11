use super::{AckedPacketHeader, HeaderParser, HeaderReader, StandardHeader};
use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use error::{FragmentErrorKind, NetworkResult};
use log::error;
use net::constants::FRAGMENT_HEADER_SIZE;
use std::io::Cursor;

#[derive(Copy, Clone, Debug)]
/// This header represents a fragmented packet header.
pub struct FragmentHeader {
    standard_header: StandardHeader,
    sequence: u16,
    id: u8,
    num_fragments: u8,
    packet_header: Option<AckedPacketHeader>,
}

impl FragmentHeader {
    /// Create new fragment with the given packet header
    pub fn new(
        standard_header: StandardHeader,
        id: u8,
        num_fragments: u8,
        packet_header: AckedPacketHeader,
    ) -> Self {
        FragmentHeader {
            standard_header,
            id,
            num_fragments,
            packet_header: Some(packet_header),
            sequence: packet_header.seq,
        }
    }

    /// Get the id of this fragment.
    pub fn id(&self) -> u8 {
        self.id
    }

    /// Get the sequence number from this packet.
    pub fn sequence(&self) -> u16 {
        self.sequence
    }

    /// Get the total number of fragments in the packet this fragment is part of.
    pub fn fragment_count(&self) -> u8 {
        self.num_fragments
    }

    /// Get the packet header if attached to fragment.
    pub fn packet_header(&self) -> Option<AckedPacketHeader> {
        self.packet_header
    }
}

impl HeaderParser for FragmentHeader {
    type Output = NetworkResult<()>;

    fn parse(&self, buffer: &mut Vec<u8>) -> <Self as HeaderParser>::Output {
        self.standard_header.parse(buffer)?;
        buffer.write_u16::<BigEndian>(self.sequence)?;
        buffer.write_u8(self.id)?;
        buffer.write_u8(self.num_fragments)?;

        // append acked header only first time
        if self.id == 0 {
            match self.packet_header {
                Some(header) => {
                    header.parse(buffer)?;
                }
                None => return Err(FragmentErrorKind::PacketHeaderNotFound.into()),
            }
        }

        Ok(())
    }
}

impl HeaderReader for FragmentHeader {
    type Header = NetworkResult<FragmentHeader>;

    fn read(rdr: &mut Cursor<&[u8]>) -> <Self as HeaderReader>::Header {
        let standard_header = StandardHeader::read(rdr)?;
        let sequence = rdr.read_u16::<BigEndian>()?;
        let id = rdr.read_u8()?;
        let num_fragments = rdr.read_u8()?;

        let mut header = FragmentHeader {
            standard_header,
            sequence,
            id,
            num_fragments,
            packet_header: None,
        };

        // append acked header is only appended to first packet.
        if id == 0 {
            header.packet_header = Some(AckedPacketHeader::read(rdr)?);
        }

        Ok(header)
    }

    /// Get the size of this header.
    fn size(&self) -> u8 {
        if self.id == 0 {
            match self.packet_header {
                Some(header) => header.size() + FRAGMENT_HEADER_SIZE,
                None => {
                    error!("Attempting to retrieve size on a 0 ID packet with no packet header");
                    0
                }
            }
        } else {
            FRAGMENT_HEADER_SIZE
        }
    }
}

#[cfg(test)]
mod tests {
    use infrastructure::DeliveryMethod;
    use packet::header::{
        AckedPacketHeader, FragmentHeader, HeaderParser, HeaderReader, StandardHeader,
    };
    use packet::PacketTypeId;
    use std::io::Cursor;

    #[test]
    pub fn serializes_deserialize_fragment_header_test() {
        // create default header
        let standard_header =
            StandardHeader::new(DeliveryMethod::UnreliableUnordered, PacketTypeId::Fragment);

        let packet_header = AckedPacketHeader::new(standard_header.clone(), 1, 1, 5421);

        // create fragment header with the default header and acked header.
        let fragment = FragmentHeader::new(standard_header.clone(), 0, 1, packet_header.clone());
        let mut fragment_buffer = Vec::with_capacity((fragment.size() + 1) as usize);
        fragment.parse(&mut fragment_buffer).unwrap();

        let mut cursor: Cursor<&[u8]> = Cursor::new(fragment_buffer.as_slice());
        let fragment_deserialized = FragmentHeader::read(&mut cursor).unwrap();

        assert_eq!(fragment_deserialized.id, 0);
        assert_eq!(fragment_deserialized.num_fragments, 1);
        assert_eq!(fragment_deserialized.sequence, 1);

        assert!(fragment_deserialized.packet_header.is_some());

        let fragment_packet_header = fragment_deserialized.packet_header.unwrap();
        assert_eq!(fragment_packet_header.seq, 1);
        assert_eq!(fragment_packet_header.ack_seq(), 1);
        assert_eq!(fragment_packet_header.ack_field(), 5421);
    }
}
