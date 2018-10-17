use super::{HeaderParser, HeaderReader};
use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use net::constants::PACKET_HEADER_SIZE;
use infrastructure::DeliveryMethod;
use packet::PacketTypeId;

use std::io::{self, Cursor, Error, ErrorKind};

#[derive(Copy, Clone, Debug)]
/// This is the default header.
pub struct PacketHeader {
    // packet id representing which type of packet this is.
    packet_type_id: PacketTypeId,
    // type representing how this packet should be delivered / processed.
    delivery_method: DeliveryMethod,
    // this is the sequence number so that we can know where in the sequence of packages this packet belongs.
    pub seq: u16,
    // this is the last acknowledged sequence number.
    ack_seq: u16,
    // this is an bitfield of all last 32 acknowledged packages
    ack_field: u32,
}

impl PacketHeader {
    /// When we compose packet headers, the local sequence becomes the sequence number of the packet, and the remote sequence becomes the ack.
    /// The ack bitfield is calculated by looking into a queue of up to 33 packets, containing sequence numbers in the range [remote sequence - 32, remote sequence].
    /// We set bit n (in [1,32]) in ack bits to 1 if the sequence number remote sequence - n is in the received queue.
    pub fn new(seq_num: u16, last_seq: u16, bit_field: u32, delivery_method: DeliveryMethod) -> PacketHeader {
        PacketHeader {
            packet_type_id: PacketTypeId::Packet,
            delivery_method,
            seq: seq_num,
            ack_seq: last_seq,
            ack_field: bit_field,
        }
    }

    /// Get the sequence number from this packet.
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

impl HeaderParser for PacketHeader {
    type Output = io::Result<Vec<u8>>;

    fn parse(&self) -> <Self as HeaderParser>::Output {
        let mut wtr = Vec::new();
        wtr.write_u8(PacketTypeId::get_id(self.packet_type_id))?;
        wtr.write_u8(DeliveryMethod::get_delivery_method_id(self.delivery_method))?;
        wtr.write_u16::<BigEndian>(self.seq)?;
        wtr.write_u16::<BigEndian>(self.ack_seq)?;
        wtr.write_u32::<BigEndian>(self.ack_field)?;
        Ok(wtr)
    }
}

impl HeaderReader for PacketHeader {
    type Header = io::Result<PacketHeader>;

    fn read(rdr: &mut Cursor<Vec<u8>>) -> <Self as HeaderReader>::Header {
        let packet_type = PacketTypeId::get_packet_type(rdr.read_u8()?);

        if packet_type != PacketTypeId::Packet {
            return Err(Error::new(ErrorKind::Other, "Invalid packet header"));
        }

        let delivery_method_id = rdr.read_u8()?;
        let seq = rdr.read_u16::<BigEndian>()?;
        let ack_seq = rdr.read_u16::<BigEndian>()?;
        let ack_field = rdr.read_u32::<BigEndian>()?;

        Ok(PacketHeader {
            packet_type_id: packet_type,
            delivery_method: DeliveryMethod::get_delivery_method_from_id(delivery_method_id),
            seq,
            ack_seq,
            ack_field,
        })
    }

    fn size(&self) -> u8 {
        PACKET_HEADER_SIZE
    }
}

mod tests {
    use byteorder::ReadBytesExt;
    use packet::header::{FragmentHeader, HeaderParser, HeaderReader, PacketHeader};
    use infrastructure::DeliveryMethod;
    use std::io::Cursor;

    #[test]
    pub fn serializes_deserialize_packet_header_test() {
        let packet_header = PacketHeader::new(1, 1, 5421, DeliveryMethod::Unreliable);
        let packet_serialized: Vec<u8> = packet_header.parse().unwrap();

        let mut cursor = Cursor::new(packet_serialized);
        let packet_deserialized: PacketHeader = PacketHeader::read(&mut cursor).unwrap();

        assert_eq!(packet_header.seq, 1);
        assert_eq!(packet_header.ack_seq, 1);
        assert_eq!(packet_header.ack_field, 5421);
    }
}
