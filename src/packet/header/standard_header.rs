use super::{HeaderReader, HeaderWriter};
use crate::error::Result;
use crate::net::constants::STANDARD_HEADER_SIZE;
use crate::packet::{DeliveryGuarantee, EnumConverter, OrderingGuarantee, PacketType};
use crate::protocol_version::ProtocolVersion;
use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use std::convert::TryFrom;
use std::io::Cursor;

#[derive(Copy, Clone, Debug)]
/// This header will be included in each packet, and contains some basic information.
pub struct StandardHeader {
    protocol_version: u16,
    packet_type: PacketType,
    delivery_guarantee: DeliveryGuarantee,
    ordering_guarantee: OrderingGuarantee,
}

impl StandardHeader {
    /// Create new header.
    pub fn new(
        delivery_guarantee: DeliveryGuarantee,
        ordering_guarantee: OrderingGuarantee,
        packet_type: PacketType,
    ) -> Self {
        StandardHeader {
            protocol_version: ProtocolVersion::get_crc16(),
            delivery_guarantee,
            ordering_guarantee,
            packet_type,
        }
    }

    /// Returns the protocol version
    #[cfg(test)]
    pub fn protocol_version(&self) -> u16 {
        self.protocol_version
    }

    /// Returns the DeliveryGuarantee
    pub fn delivery_guarantee(&self) -> DeliveryGuarantee {
        self.delivery_guarantee
    }

    /// Returns the OrderingGuarantee
    pub fn ordering_guarantee(&self) -> OrderingGuarantee {
        self.ordering_guarantee
    }

    /// Returns the PacketType
    #[cfg(test)]
    pub fn packet_type(&self) -> PacketType {
        self.packet_type
    }

    /// Returns true if the packet is a heartbeat packet, false otherwise
    pub fn is_heartbeat(&self) -> bool {
        self.packet_type == PacketType::Heartbeat
    }

    /// Returns true if the packet is a fragment, false if not
    pub fn is_fragment(&self) -> bool {
        self.packet_type == PacketType::Fragment
    }

    /// Checks if the protocol version in the packet is a valid version
    pub fn is_current_protocol(&self) -> bool {
        ProtocolVersion::valid_version(self.protocol_version)
    }
}

impl Default for StandardHeader {
    fn default() -> Self {
        StandardHeader::new(
            DeliveryGuarantee::Unreliable,
            OrderingGuarantee::None,
            PacketType::Packet,
        )
    }
}

impl HeaderWriter for StandardHeader {
    type Output = Result<()>;

    fn parse(&self, buffer: &mut Vec<u8>) -> Self::Output {
        buffer.write_u16::<BigEndian>(self.protocol_version)?;
        buffer.write_u8(self.packet_type.to_u8())?;
        buffer.write_u8(self.delivery_guarantee.to_u8())?;
        buffer.write_u8(self.ordering_guarantee.to_u8())?;
        Ok(())
    }
}

impl HeaderReader for StandardHeader {
    type Header = Result<StandardHeader>;

    fn read(rdr: &mut Cursor<&[u8]>) -> Self::Header {
        let protocol_version = rdr.read_u16::<BigEndian>()?; /* protocol id */
        let packet_id = rdr.read_u8()?;
        let delivery_guarantee_id = rdr.read_u8()?;
        let order_guarantee_id = rdr.read_u8()?;

        let header = StandardHeader {
            protocol_version,
            packet_type: PacketType::try_from(packet_id)?,
            delivery_guarantee: DeliveryGuarantee::try_from(delivery_guarantee_id)?,
            ordering_guarantee: OrderingGuarantee::try_from(order_guarantee_id)?,
        };

        Ok(header)
    }

    /// Get the size of this header.
    fn size() -> u8 {
        STANDARD_HEADER_SIZE
    }
}

#[cfg(test)]
mod tests {
    use crate::net::constants::STANDARD_HEADER_SIZE;
    use crate::packet::header::{HeaderReader, HeaderWriter, StandardHeader};
    use crate::packet::{DeliveryGuarantee, EnumConverter, OrderingGuarantee, PacketType};
    use std::io::Cursor;

    #[test]
    fn serialize() {
        let mut buffer = Vec::new();
        let header = StandardHeader::new(
            DeliveryGuarantee::Unreliable,
            OrderingGuarantee::Sequenced(None),
            PacketType::Packet,
        );
        header.parse(&mut buffer).is_ok();

        // [0 .. 3] protocol version
        assert_eq!(buffer[2], PacketType::Packet.to_u8());
        assert_eq!(buffer[3], DeliveryGuarantee::Unreliable.to_u8());
        assert_eq!(buffer[4], OrderingGuarantee::Sequenced(None).to_u8());
    }

    #[test]
    fn deserialize() {
        let buffer = vec![0, 1, 0, 1, 1];

        let mut cursor = Cursor::new(buffer.as_slice());

        let header = StandardHeader::read(&mut cursor).unwrap();

        assert_eq!(header.protocol_version(), 1);
        assert_eq!(header.packet_type(), PacketType::Packet);
        assert_eq!(header.delivery_guarantee(), DeliveryGuarantee::Reliable);
        assert_eq!(
            header.ordering_guarantee(),
            OrderingGuarantee::Sequenced(None)
        );
    }

    #[test]
    fn size() {
        assert_eq!(StandardHeader::size(), STANDARD_HEADER_SIZE);
    }
}
