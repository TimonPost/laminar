use super::{HeaderReader, HeaderWriter};
use crate::error::Result;
use crate::net::constants::HEART_BEAT_HEADER_SIZE;
use crate::packet::PacketTypeId;
use crate::protocol_version::ProtocolVersion;
use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use std::io::Cursor;

#[derive(Copy, Clone, Debug)]
/// This header represents an heartbeat packet header.
/// An heart beat just keeps the client awake.
pub struct HeartBeatHeader {
    packet_type_id: PacketTypeId,
}

impl HeartBeatHeader {
    /// Create new heartbeat header.
    pub fn new() -> Self {
        HeartBeatHeader {
            packet_type_id: PacketTypeId::HeartBeat,
        }
    }
}

impl Default for HeartBeatHeader {
    fn default() -> Self {
        HeartBeatHeader::new()
    }
}

impl HeaderWriter for HeartBeatHeader {
    type Output = Result<()>;

    fn parse(&self, buffer: &mut Vec<u8>) -> Self::Output {
        buffer.write_u16::<BigEndian>(ProtocolVersion::get_crc16())?;
        buffer.write_u8(PacketTypeId::get_id(self.packet_type_id))?;

        Ok(())
    }
}

impl HeaderReader for HeartBeatHeader {
    type Header = Result<HeartBeatHeader>;

    fn read(rdr: &mut Cursor<&[u8]>) -> Self::Header {
        let _ = rdr.read_u32::<BigEndian>()?;
        let _ = rdr.read_u8();
        let header = HeartBeatHeader {
            packet_type_id: PacketTypeId::HeartBeat,
        };

        Ok(header)
    }

    /// Get the size of this header.
    fn size(&self) -> u8 {
        HEART_BEAT_HEADER_SIZE
    }
}
