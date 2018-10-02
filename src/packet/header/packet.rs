use std::io::{self, Cursor};
use super::{HeaderParser, HeaderReader};
use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};

#[derive(Copy, Clone, Debug)]
/// This is the default header.
pub struct PacketHeader
{
    // this is the sequence number so that we can know where in the sequence of packages this packet belongs.
    pub seq: u16,
    // this is the last acknowledged sequence number.
    pub ack_seq: u16,
    // this is an bitfield of all last 32 acknowledged packages
    pub ack_field: u32,
}

impl PacketHeader {
    pub fn new(seq_num: u16, last_seq: u16, bit_field: u32) -> PacketHeader {
        PacketHeader {
            seq: seq_num,
            ack_seq: last_seq,
            ack_field: bit_field,
        }
    }

    /// Get the size of this header.
    pub fn size(&self) -> u8
    {
        return 8;
    }
}

impl HeaderParser for PacketHeader
{
    type Output =  io::Result<Vec<u8>>;

    fn parse(&self) -> <Self as HeaderParser>::Output {
        let mut wtr = Vec::new();
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
        let seq = rdr.read_u16::<BigEndian>()?;
        let ack_seq = rdr.read_u16::<BigEndian>()?;
        let ack_field = rdr.read_u32::<BigEndian>()?;

        Ok(PacketHeader {
            seq,
            ack_seq,
            ack_field,
        })
    }
}