use super::{HeaderParser, HeaderReader};
use super::PacketHeader;
use error::{Result, NetworkError};

use std::io::{self, Cursor, Error, ErrorKind, Write};
use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};

#[derive(Copy, Clone, Debug)]
/// This header represents an fragmented packet header.
pub struct FragmentHeader {
    pub sequence: u16,
    pub id: u8,
    pub num_fragments: u8,
    pub packet_header: Option<PacketHeader>,
}

impl FragmentHeader {
    /// Create new fragment with the given packet header
    pub fn new(id: u8, num_fragments: u8, packet_header: PacketHeader) -> Self {
        FragmentHeader { id, num_fragments, packet_header: Some(packet_header), sequence: packet_header.seq }
    }

    /// Get the size of this header.
    pub fn size(&self) -> u8
    {
        if self.id == 0 {
            if self.packet_header.is_some() {
                return self.packet_header.as_ref().unwrap().size() + 5;
            }else {
                error!("Attemtping to retrieve size on a 0 ID packet with no packet header");
                0
            }
        } else {
            5
        }
    }
}

impl HeaderParser for FragmentHeader
{
    type Output = io::Result<Vec<u8>>;

    fn parse(&self) -> <Self as HeaderParser>::Output {
        let mut wtr = Vec::new();
        wtr.write_u8(1)?;
        wtr.write_u16::<BigEndian>(self.sequence)?;
        wtr.write_u8(self.id)?;
        wtr.write_u8(self.num_fragments)?;

        if self.id == 0 {
            if self.packet_header.is_some() {
                wtr.write(&self.packet_header.as_ref().unwrap().parse()?);
            } else {
                return Err(Error::new(ErrorKind::Other, "Expected packet header to be some but was instead non"));
            }
        }

        Ok(wtr)
    }
}

impl HeaderReader for FragmentHeader
{
    type Header =  Result<FragmentHeader>;

    fn read(rdr: &mut Cursor<Vec<u8>>) -> <Self as HeaderReader>::Header {
        let prefix_byte = rdr.read_u8()?;

        if prefix_byte != 1 {
            return Err(NetworkError::FragmentInvalid.into())
        }

        let sequence = rdr.read_u16::<BigEndian>()?;
        let id = rdr.read_u8()?;
        let num_fragments = rdr.read_u8()?;

        let mut header = FragmentHeader {
            sequence,
            id,
            num_fragments,
            packet_header: None
        };

        if id == 0 {
            header.packet_header = Some(PacketHeader::read(rdr)?);
        }

        Ok(header)
    }
}