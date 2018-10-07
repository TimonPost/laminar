use std::io::{self, Cursor, Error, ErrorKind};
use super::{HeaderParser, HeaderReader};
use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use net::constants::PACKET_HEADER_SIZE;


#[derive(Copy, Clone, Debug)]
/// This is the default header.
pub struct PacketHeader
{
    // this is the sequence number so that we can know where in the sequence of packages this packet belongs.
    pub seq: u16,
    // this is the last acknowledged sequence number.
    ack_seq: u16,
    // this is an bitfield of all last 32 acknowledged packages
    ack_field: u32,
}

impl PacketHeader {
    pub fn new(seq_num: u16, last_seq: u16, bit_field: u32) -> PacketHeader {
        PacketHeader {
            seq: seq_num,
            ack_seq: last_seq,
            ack_field: bit_field,
        }
    }

    /// Get the sequence number from this packet.
    pub fn sequence(&self) -> u16
    {
        self.sequence()
    }

    /// Get bit field of all last 32 acknowledged packages
    pub fn ack_field(&self) -> u32
    {
        self.ack_field
    }

    /// Get last acknowledged sequence number.
    pub fn ack_seq(&self) -> u16
    {
        self.ack_seq
    }
}

impl HeaderParser for PacketHeader
{
    type Output =  io::Result<Vec<u8>>;

    fn parse(&self) -> <Self as HeaderParser>::Output {
        let mut wtr = Vec::new();
        wtr.write_u8(0)?;
        wtr.write_u16::<BigEndian>(self.seq)?;
        wtr.write_u16::<BigEndian>(self.ack_seq)?;
        wtr.write_u32::<BigEndian>(self.ack_field)?;
        Ok(wtr)
    }
}

impl HeaderReader for PacketHeader
{
    type Header = io::Result<PacketHeader>;

    fn read(rdr:  &mut Cursor<Vec<u8>>) -> <Self as HeaderReader>::Header {
        let prefix_byte = rdr.read_u8()?;

        if prefix_byte != 0 {
            return  Err(Error::new(ErrorKind::Other, "Invalid packet header"));
        }

        let seq = rdr.read_u16::<BigEndian>()?;
        let ack_seq = rdr.read_u16::<BigEndian>()?;
        let ack_field = rdr.read_u32::<BigEndian>()?;

        Ok(PacketHeader {
            seq,
            ack_seq,
            ack_field,
        })
    }

    fn size(&self) -> u8
    {
        PACKET_HEADER_SIZE
    }
}

mod tests
{
    use packet::header::{PacketHeader, FragmentHeader, HeaderParser, HeaderReader};
    use byteorder::ReadBytesExt;
    use std::io::Cursor;

    #[test]
    pub fn serializes_deserialize_packet_header_test()
    {
        let packet_header = PacketHeader::new(1,1,5421);
        let packet_serialized: Vec<u8> = packet_header.parse().unwrap();

        let mut cursor = Cursor::new(packet_serialized);
        let packet_deserialized: PacketHeader = PacketHeader::read(&mut cursor).unwrap();

        assert_eq!(packet_header.seq, 1);
        assert_eq!(packet_header.ack_seq, 1);
        assert_eq!(packet_header.ack_field, 5421);
    }
}