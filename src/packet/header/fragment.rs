use super::PacketHeader;
use super::{HeaderParser, HeaderReader};
use net::constants::FRAGMENT_HEADER_SIZE;
use packet::PacketTypeId;
use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use std::io::{self, Cursor, Error, ErrorKind, Write};

#[derive(Copy, Clone, Debug)]
/// This header represents an fragmented packet header.
pub struct FragmentHeader {
    packet_type_id: PacketTypeId,
    sequence: u16,
    id: u8,
    num_fragments: u8,
    packet_header: Option<PacketHeader>,
}

impl FragmentHeader {
    /// Create new fragment with the given packet header
    pub fn new(id: u8, num_fragments: u8, packet_header: PacketHeader) -> Self {
        FragmentHeader {
            packet_type_id: PacketTypeId::Fragment,
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

    /// Get the total number of fragments from an packet this fragment is part of.
    pub fn fragment_count(&self) -> u8 {
        self.num_fragments
    }

    /// Get the packet header if attached to fragment.
    pub fn packet_header(&self) -> Option<PacketHeader> {
        self.packet_header
    }
}

impl HeaderParser for FragmentHeader {
    type Output = io::Result<Vec<u8>>;

    fn parse(&self) -> <Self as HeaderParser>::Output {
        let mut wtr = Vec::new();
        wtr.write_u8(PacketTypeId::get_id(self.packet_type_id))?;
        wtr.write_u16::<BigEndian>(self.sequence)?;
        wtr.write_u8(self.id)?;
        wtr.write_u8(self.num_fragments)?;

        if self.id == 0 {
            match self.packet_header {
                Some(header) => {
                    wtr.write(&header.parse()?)?;
                }
                None => return Err(Error::new(ErrorKind::Other, "Invalid fragment header")),
            }
        }

        Ok(wtr)
    }
}

impl HeaderReader for FragmentHeader {
    type Header = io::Result<FragmentHeader>;

    fn read(rdr: &mut Cursor<Vec<u8>>) -> <Self as HeaderReader>::Header {
        let packet_type_id = PacketTypeId::get_packet_type(rdr.read_u8()?);

        if packet_type_id != PacketTypeId::Fragment {
            return Err(Error::new(ErrorKind::Other, "Invalid fragment header"));
        }

        let sequence = rdr.read_u16::<BigEndian>()?;
        let id = rdr.read_u8()?;
        let num_fragments = rdr.read_u8()?;

        let mut header = FragmentHeader {
            packet_type_id,
            sequence,
            id,
            num_fragments,
            packet_header: None,
        };

        if id == 0 {
            header.packet_header = Some(PacketHeader::read(rdr)?);
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

mod tests {
    use byteorder::ReadBytesExt;
    use packet::header::{FragmentHeader, HeaderParser, HeaderReader, PacketHeader};
    use infrastructure::DeliveryMethod;
    use std::io::Cursor;

    #[test]
    pub fn serializes_deserialize_fragment_header_test() {
        let packet_header = PacketHeader::new(1, 1, 5421, DeliveryMethod::Unreliable);
        let packet_serialized: Vec<u8> = packet_header.parse().unwrap();

        let fragment = FragmentHeader::new(0, 1, packet_header.clone());
        let fragment_serialized = fragment.parse().unwrap();

        let mut cursor = Cursor::new(fragment_serialized);
        let fragment_deserialized: FragmentHeader = FragmentHeader::read(&mut cursor).unwrap();

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
