use super::PacketHeader;
use super::{HeaderParser, HeaderReader};
use net::constants::{FRAGMENT_HEADER_SIZE, HEART_BEAT_HEADER_SIZE};
use packet::PacketTypeId;
use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use std::io::{self, Cursor, Error, ErrorKind, Write};

#[derive(Copy, Clone, Debug)]
/// This header represents an heartbeat packet header.
/// An heart beat just keeps the client awake.
pub struct HeartBeatHeader {
    packet_type_id: PacketTypeId
}

impl HeartBeatHeader {
    /// Create new heartbeat header.
    pub fn new() -> Self {
        HeartBeatHeader {
            packet_type_id: PacketTypeId::HeartBeat
        }
    }
}

impl HeaderParser for HeartBeatHeader {
    type Output = io::Result<Vec<u8>>;

    fn parse(&self) -> <Self as HeaderParser>::Output {
        let mut wtr = Vec::new();
        wtr.write_u8(PacketTypeId::get_id(self.packet_type_id))?;

        Ok(wtr)
    }
}

impl HeaderReader for HeartBeatHeader {
    type Header = io::Result<HeartBeatHeader>;

    fn read(rdr: &mut Cursor<Vec<u8>>) -> <Self as HeaderReader>::Header {
        let packet_type_id = PacketTypeId::get_packet_type(rdr.read_u8()?);

        if packet_type_id != PacketTypeId::HeartBeat {
            return Err(Error::new(ErrorKind::Other, "Invalid fragment header"));
        }

        let mut header = HeartBeatHeader {
           packet_type_id
        };

        Ok(header)
    }

    /// Get the size of this header.
    fn size(&self) -> u8 {
       return HEART_BEAT_HEADER_SIZE;
    }
}


