use super::{HeaderWriter, HeaderReader};
use crate::error::NetworkResult;
use crate::infrastructure::DeliveryMethod;
use crate::net::constants::STANDARD_HEADER_SIZE;
use crate::packet::PacketTypeId;
use crate::protocol_version::ProtocolVersion;
use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use std::io::Cursor;

#[derive(Copy, Clone, Debug)]
/// This header will be included in each packet, and contains some basic information.
pub struct StandardHeader {
    /// crc32 of the protocol version.
    pub protocol_version: u32,
    /// specifies the packet type.
    pub packet_type_id: PacketTypeId,
    /// specifies how this packet should be processed.
    pub delivery_method: DeliveryMethod,
}

impl StandardHeader {
    /// Create new heartbeat header.
    pub fn new(delivery_method: DeliveryMethod, packet_type_id: PacketTypeId) -> Self {
        StandardHeader {
            protocol_version: ProtocolVersion::get_crc32(),
            packet_type_id,
            delivery_method,
        }
    }
}

impl Default for StandardHeader {
    fn default() -> Self {
        StandardHeader::new(DeliveryMethod::UnreliableUnordered, PacketTypeId::Packet)
    }
}

impl HeaderWriter for StandardHeader {
    type Output = NetworkResult<()>;

    fn parse(&self, buffer: &mut Vec<u8>) -> <Self as HeaderWriter>::Output {
        buffer.write_u32::<BigEndian>(self.protocol_version)?;
        buffer.write_u8(PacketTypeId::get_id(self.packet_type_id))?;
        buffer.write_u8(DeliveryMethod::get_delivery_method_id(self.delivery_method))?;

        Ok(())
    }
}

impl HeaderReader for StandardHeader {
    type Header = NetworkResult<StandardHeader>;

    fn read(rdr: &mut Cursor<&[u8]>) -> <Self as HeaderReader>::Header {
        let protocol_version = rdr.read_u32::<BigEndian>()?; /* protocol id */
        let packet_id = rdr.read_u8()?;
        let delivery_method_id = rdr.read_u8()?;

        let header = StandardHeader {
            protocol_version,
            packet_type_id: PacketTypeId::get_packet_type(packet_id),
            delivery_method: DeliveryMethod::get_delivery_method_from_id(delivery_method_id),
        };

        Ok(header)
    }

    /// Get the size of this header.
    fn size(&self) -> u8 {
        STANDARD_HEADER_SIZE
    }
}

#[cfg(test)]
mod tests {
    use crate::infrastructure::DeliveryMethod;
    use crate::packet::header::{HeaderWriter, HeaderReader, StandardHeader};
    use crate::packet::PacketTypeId;
    use crate::protocol_version::ProtocolVersion;
    use std::io::Cursor;

    #[test]
    pub fn serializes_deserialize_packet_header_test() {
        let packet_header = StandardHeader::default();
        let mut buffer = Vec::with_capacity((packet_header.size() + 1) as usize);

        let _ = packet_header.parse(&mut buffer);

        let mut cursor = Cursor::new(buffer.as_slice());
        let packet_header = StandardHeader::read(&mut cursor).unwrap();
        assert!(ProtocolVersion::valid_version(
            packet_header.protocol_version
        ));
        assert_eq!(packet_header.packet_type_id, PacketTypeId::Packet);
        assert_eq!(
            packet_header.delivery_method,
            DeliveryMethod::UnreliableUnordered
        );
    }
}
