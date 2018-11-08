use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};

use super::{HeaderParser, HeaderReader};

use net::constants::HEART_BEAT_HEADER_SIZE;
use protocol_version::ProtocolVersion;
use error::NetworkResult;
use packet::PacketTypeId;

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

impl HeaderParser for HeartBeatHeader {
    type Output = NetworkResult<()>;

    fn parse(&self, buffer: &mut Vec<u8>) -> <Self as HeaderParser>::Output {
        buffer.write_u32::<BigEndian>(ProtocolVersion::get_crc32())?;
        buffer.write_u8(PacketTypeId::get_id(self.packet_type_id))?;

        Ok(())
    }
}

impl HeaderReader for HeartBeatHeader {
    type Header = NetworkResult<HeartBeatHeader>;

    fn read(rdr: &mut Cursor<&[u8]>) -> <Self as HeaderReader>::Header {
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
